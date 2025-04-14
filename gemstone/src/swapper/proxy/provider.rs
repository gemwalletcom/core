use alloy_primitives::U256;
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::Arc;

use super::{
    client::ProxyClient,
    mayan::{MayanClientStatus, MayanExplorer},
};
use crate::{
    config::swap_config::DEFAULT_SWAP_FEE_BPS,
    network::AlienProvider,
    swapper::{
        approval::check_approval_erc20,
        models::{ApprovalData, ApprovalType, SwapChainAsset},
        FetchQuoteData, GemSwapProvider, SwapProviderData, SwapProviderType, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, Swapper, SwapperError,
    },
};
use primitives::{Chain, ChainType};
use swap_primitives::{Quote, QuoteData, QuoteRequest, ReferralAddress, ReferralInfo};

pub const PROVIDER_API_URL: &str = "https://api.gemwallet.com/swapper";
const DEFAULT_GAS_LIMIT: u64 = 500000;

#[derive(Debug)]
pub struct ProxyProvider {
    pub provider: SwapProviderType,
    pub url: String,
    pub assets: Vec<SwapChainAsset>,
}

impl ProxyProvider {
    pub async fn check_approval(
        &self,
        quote: &SwapQuote,
        quote_data: &QuoteData,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<(Option<ApprovalData>, Option<String>), SwapperError> {
        let request = &quote.request;
        let from_asset = &request.from_asset;

        if from_asset.chain().chain_type() != ChainType::Ethereum || from_asset.is_native() {
            return Ok((None, None));
        }

        let token = from_asset.id.token_id.clone().unwrap();
        let wallet_address = request.wallet_address.clone();
        let spender = quote_data.to.clone();
        let amount = U256::from_str(&quote.from_value).map_err(SwapperError::from)?;
        let approval = check_approval_erc20(wallet_address, token, spender.to_string(), amount, provider, &request.from_asset.chain()).await?;

        let gas_limit: Option<String> = if matches!(approval, ApprovalType::Approve(_)) {
            Some(DEFAULT_GAS_LIMIT.to_string())
        } else {
            None
        };

        Ok((approval.approval_data(), gas_limit))
    }
}

#[async_trait]
impl Swapper for ProxyProvider {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        self.assets.clone()
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let client = ProxyClient::new(provider);
        let referral = request.options.fee.clone().unwrap_or_default();
        let quote_request = QuoteRequest {
            from_address: request.wallet_address.clone(),
            to_address: request.destination_address.clone(),
            from_asset: request.from_asset.clone(),
            to_asset: request.to_asset.clone(),
            from_value: request.value.clone(),
            referral: ReferralInfo {
                address: ReferralAddress {
                    evm: Some(referral.evm.address.clone()),
                    solana: Some(referral.solana.address.clone()),
                    sui: Some(referral.sui.address.clone()),
                    ton: Some(referral.ton.address.clone()),
                    tron: Some(referral.tron.address.clone()),
                },
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            slippage_bps: request.options.slippage.bps,
        };

        let quote = client.get_quote(&self.url, quote_request.clone()).await?;

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: quote.output_value.clone(),
            data: SwapProviderData {
                provider: self.provider().clone(),
                routes: vec![SwapRoute {
                    input: request.from_asset.id.clone(),
                    output: request.to_asset.id.clone(),
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

    async fn get_transaction_status(&self, _chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        match self.provider.id {
            GemSwapProvider::Mayan => {
                let client = MayanExplorer::new(provider);
                let result = client.get_transaction_status(transaction_hash).await?;
                Ok(result.client_status == MayanClientStatus::Completed)
            }
            _ => Ok(true),
        }
    }
}
