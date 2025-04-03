mod client;
mod model;

use alloy_primitives::U256;
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::Arc;

use super::{
    approval::check_approval_erc20, FetchQuoteData, GemSwapOptions, GemSwapProvider, SwapProvider, SwapProviderData, SwapProviderType, SwapQuote,
    SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError,
};
use crate::{
    config::swap_config::SwapReferralFee,
    network::AlienProvider,
    swapper::{
        asset::*,
        models::{ApprovalData, ApprovalType, SwapChainAsset},
    },
};
use client::ProxyClient;
use model::{Quote, QuoteData, QuoteRequest};
use primitives::{Chain, ChainType};

const PROVIDER_URL: &str = "https://api.gemwallet.com/swapper";
const DEFAULT_GAS_LIMIT: u64 = 500000;

#[derive(Debug)]
pub struct ProxyProvider {
    pub provider: SwapProviderType,
    pub url: String,
}

impl ProxyProvider {
    pub fn new_stonfi_v2() -> Self {
        Self {
            provider: SwapProviderType::new(SwapProvider::StonFiV2),
            url: format!("{}/{}", PROVIDER_URL, "stonfi_v2"),
        }
    }

    pub fn new_mayan() -> Self {
        Self {
            provider: SwapProviderType::new(SwapProvider::Mayan),
            url: format!("{}/{}", PROVIDER_URL, "mayan"),
        }
    }

    fn get_referrer(&self, chain: &Chain, options: &GemSwapOptions, provider: &SwapProvider) -> SwapReferralFee {
        match provider {
            // always use solana for Mayan, otherwise not supported chain error
            SwapProvider::Mayan => {
                return options.fee.as_ref().unwrap().solana.clone();
            }
            _ => {}
        }

        match chain.chain_type() {
            ChainType::Ethereum => options.fee.as_ref().unwrap().evm.clone(),
            ChainType::Solana => options.fee.as_ref().unwrap().solana.clone(),
            ChainType::Ton => options.fee.as_ref().unwrap().ton.clone(),
            ChainType::Sui => options.fee.as_ref().unwrap().sui.clone(),
            _ => SwapReferralFee::default(),
        }
    }

    pub async fn check_approval(
        &self,
        quote: &SwapQuote,
        quote_data: &QuoteData,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<(Option<ApprovalData>, Option<String>), SwapperError> {
        let request = &quote.request;
        let from_asset = &request.from_asset;

        if from_asset.chain.chain_type() != ChainType::Ethereum || from_asset.is_native() {
            return Ok((None, None));
        }

        let token = from_asset.token_id.clone().unwrap();
        let wallet_address = request.wallet_address.clone();
        let spender = quote_data.to.clone();
        let amount = U256::from_str(&quote.from_value).map_err(|_| SwapperError::InvalidAmount)?;
        let approval = check_approval_erc20(wallet_address, token, spender.to_string(), amount, provider, &request.from_asset.chain).await?;

        let gas_limit: Option<String> = if matches!(approval, ApprovalType::Approve(_)) {
            Some(DEFAULT_GAS_LIMIT.to_string())
        } else {
            None
        };

        Ok((approval.approval_data(), gas_limit))
    }
}

#[async_trait]
impl GemSwapProvider for ProxyProvider {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        vec![
            // StoneFiV2
            SwapChainAsset::All(Chain::Ton),
            // Mayan
            SwapChainAsset::Assets(
                Chain::Ethereum,
                vec![
                    ETHEREUM_USDT.id.clone(),
                    ETHEREUM_USDC.id.clone(),
                    ETHEREUM_DAI.id.clone(),
                    ETHEREUM_USDS.id.clone(),
                    ETHEREUM_WBTC.id.clone(),
                    ETHEREUM_WETH.id.clone(),
                    ETHEREUM_STETH.id.clone(),
                    ETHEREUM_CBBTC.id.clone(),
                ],
            ),
            SwapChainAsset::Assets(
                Chain::Solana,
                vec![
                    SOLANA_USDC.id.clone(),
                    SOLANA_USDT.id.clone(),
                    SOLANA_USDS.id.clone(),
                    SOLANA_CBBTC.id.clone(),
                    SOLANA_WBTC.id.clone(),
                    SOLANA_JITO_SOL.id.clone(),
                ],
            ),
            SwapChainAsset::Assets(Chain::Sui, vec![SUI_USDC.id.clone(), SUI_SBUSDT.id.clone(), SUI_WAL.id.clone()]),
            SwapChainAsset::Assets(
                Chain::SmartChain,
                vec![SMARTCHAIN_USDT.id.clone(), SMARTCHAIN_USDC.id.clone(), SMARTCHAIN_WBTC.id.clone()],
            ),
            SwapChainAsset::Assets(
                Chain::Base,
                vec![BASE_USDC.id.clone(), BASE_CBBTC.id.clone(), BASE_WBTC.id.clone(), BASE_USDS.id.clone()],
            ),
            SwapChainAsset::Assets(Chain::Polygon, vec![POLYGON_USDC.id.clone(), POLYGON_USDT.id.clone()]),
            SwapChainAsset::Assets(Chain::AvalancheC, vec![AVALANCHE_USDT.id.clone(), AVALANCHE_USDC.id.clone()]),
        ]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let client = ProxyClient::new(provider);
        let referrer = self.get_referrer(&request.from_asset.chain, &request.options, &self.provider.id);
        let quote_request = QuoteRequest {
            from_address: request.wallet_address.clone(),
            from_asset: request.from_asset.to_string(),
            to_address: request.destination_address.clone(),
            to_asset: request.to_asset.to_string(),
            from_value: request.value.clone(),
            referral_address: referrer.address.clone(),
            referral_bps: referrer.bps as usize,
            slippage_bps: request.options.slippage.bps as usize,
        };

        let quote = client.get_quote(&self.url, quote_request.clone()).await?;

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: quote.output_value.clone(),
            to_min_value: quote.output_value.clone(),
            data: SwapProviderData {
                provider: self.provider().clone(),
                routes: vec![SwapRoute {
                    input: request.from_asset.clone(),
                    output: request.to_asset.clone(),
                    route_data: serde_json::to_string(&quote).unwrap(),
                    gas_limit: None,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let routes = quote.data.clone().routes;
        let route_data: Quote = serde_json::from_str(&routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let client = ProxyClient::new(provider.clone());

        let data = client.get_quote_data(&self.url, route_data).await?;
        let (approval, gas_limit) = self.check_approval(quote, &data, provider).await?;

        Ok(SwapQuoteData {
            to: data.to,
            value: data.value,
            data: data.data,
            approval,
            gas_limit,
        })
    }
}
