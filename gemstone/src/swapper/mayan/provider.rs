use alloy_core::primitives::{Address, U256};
use async_trait::async_trait;
use gem_solana::WSOL_TOKEN_ADDRESS;
use num_bigint::BigInt;
use std::{str::FromStr, sync::Arc};

use super::{
    models::{Quote, QuoteOptions, QuoteType, QuoteUrlParams},
    relayer::MayanRelayer,
    tx_builder::MayanTxBuilder,
    MAYAN_FORWARDER_CONTRACT,
};
use crate::{
    config::swap_config::SwapReferralFee,
    network::AlienProvider,
    swapper::{
        approval::{check_approval, CheckApprovalType},
        ApprovalType, FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider, SwapProviderData, SwapProviderType, SwapQuote, SwapQuoteData,
        SwapQuoteRequest, SwapRoute, SwapperError,
    },
};
use gem_evm::ether_conv;
use primitives::{AssetId, Chain, ChainType};

#[derive(Debug)]
pub struct MayanSwiftProvider {
    provider: SwapProviderType,
}

impl Default for MayanSwiftProvider {
    fn default() -> Self {
        Self {
            provider: SwapProviderType::new(SwapProvider::MayanSwift),
        }
    }
}

impl MayanSwiftProvider {
    fn get_referrer(&self, request: &SwapQuoteRequest) -> Result<Option<SwapReferralFee>, SwapperError> {
        if let Some(fees) = &request.options.fee {
            return Ok(Some(fees.solana.clone()));
        }
        Ok(None)
    }

    pub async fn check_approval(request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<ApprovalType, SwapperError> {
        if request.from_asset.is_native() {
            return Ok(ApprovalType::None);
        }

        check_approval(
            CheckApprovalType::ERC20 {
                owner: request.wallet_address.clone(),
                token: request.from_asset.token_id.clone().unwrap_or_default(),
                spender: MAYAN_FORWARDER_CONTRACT.to_string(),
                amount: U256::from_str(request.value.as_str()).map_err(|_| SwapperError::InvalidAmount)?,
            },
            provider,
            &request.from_asset.chain,
        )
        .await
    }

    fn normalize_address(asset_id: &AssetId) -> Result<String, SwapperError> {
        if let Some(token_id) = &asset_id.token_id {
            Ok(token_id.clone())
        } else {
            match asset_id.chain.chain_type() {
                ChainType::Ethereum => Ok(Address::ZERO.to_string()),
                ChainType::Solana => Ok(WSOL_TOKEN_ADDRESS.to_string()),
                _ => Err(SwapperError::NotSupportedAsset),
            }
        }
    }

    async fn fetch_quote_from_request(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<Quote, SwapperError> {
        let mayan_relayer = MayanRelayer::default_relayer(provider.clone());
        let referrer = self.get_referrer(request)?;

        let from_token = Self::normalize_address(&request.from_asset)?;
        let to_token = Self::normalize_address(&request.to_asset)?;

        let amount = &request.value.parse::<BigInt>()?;

        let quote_params = QuoteUrlParams::new(
            amount.to_string(),
            from_token,
            request.from_asset.chain,
            to_token,
            request.to_asset.chain,
            &QuoteOptions::default(),
            Some("auto".to_string()),
            referrer.clone().map(|x| x.address),
            referrer.map(|x| x.bps),
        );

        let quote = mayan_relayer.get_quote(quote_params).await.map_err(|e| SwapperError::ComputeQuoteError {
            msg: format!("Mayan relayer quote error: {:?}", e),
        })?;

        // TODO: adjust to find most effective quote
        let most_effective_quote = quote.into_iter().filter(|x| x.r#type == QuoteType::Swift.to_string()).last();

        most_effective_quote.ok_or(SwapperError::NoQuoteAvailable)
    }
}

#[async_trait]
impl GemSwapProvider for MayanSwiftProvider {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        vec![
            SwapChainAsset::All(Chain::Solana),
            SwapChainAsset::All(Chain::Polygon),
            SwapChainAsset::All(Chain::AvalancheC),
            SwapChainAsset::All(Chain::SmartChain),
            SwapChainAsset::All(Chain::Ethereum),
        ]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        if request.from_asset.chain.chain_type() == ChainType::Solana {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let quote = self.fetch_quote_from_request(request, provider.clone()).await?;
        let serialized_quote = serde_json::to_string(&quote).map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;
        let to_value = ether_conv::to_bn_wei(&quote.expected_amount_out.to_string(), quote.to_token.decimals as u32).to_string();
        let to_min_value = ether_conv::to_bn_wei(&quote.min_amount_out.to_string(), quote.to_token.decimals as u32).to_string();

        Ok(SwapQuote {
            from_value: quote.effective_amount_in64.to_string(),
            to_value,
            to_min_value,
            data: SwapProviderData {
                provider: self.provider.clone(),
                slippage_bps: quote.slippage_bps,
                routes: vec![SwapRoute {
                    route_data: serialized_quote,
                    input: request.from_asset.clone(),
                    output: request.to_asset.clone(),
                    gas_limit: None,
                }],
            },
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let request = &quote.request;
        let route_data = &quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?.route_data;
        let mayan_quote: Quote = serde_json::from_str(route_data).map_err(|_| SwapperError::InvalidRoute)?;

        let referrer = self.get_referrer(request)?;
        let tx_builder = MayanTxBuilder::default();
        if request.from_asset.chain.chain_type() == ChainType::Ethereum {
            let approval = Self::check_approval(request, provider.clone()).await?;

            tx_builder.build_evm_tx(
                mayan_quote,
                approval.approval_data(),
                &request.wallet_address,
                &request.destination_address,
                referrer,
            )
        } else if request.from_asset.chain.chain_type() == ChainType::Solana {
            tx_builder.build_sol_tx(mayan_quote)
        } else {
            return Err(SwapperError::InvalidRoute);
        }
    }
}
