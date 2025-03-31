use std::sync::Arc;

mod client;
mod model;

use crate::{network::AlienProvider, swapper::SwapChainAsset};
use async_trait::async_trait;
use client::ProxyClient;
use model::{Quote, QuoteRequest};
use primitives::Chain;

use super::{
    FetchQuoteData, GemSwapProvider, SwapProvider, SwapProviderData, SwapProviderType, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError,
};

const PROVIDER_URL: &str = "https://api.gemwallet.com/swapper";

#[derive(Debug)]
pub struct ProxyProvider {
    pub provider: SwapProviderType,
    pub chain: Chain,
    pub url: String,
}

impl ProxyProvider {
    pub fn new_stonfi_v2() -> Self {
        Self {
            provider: SwapProviderType::new(SwapProvider::StonFiV2),
            chain: Chain::Ton,
            url: format!("{}/{}", PROVIDER_URL, "stonfi_v2"),
        }
    }
}

#[async_trait]
impl GemSwapProvider for ProxyProvider {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        vec![SwapChainAsset::All(Chain::Ton)]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let client = ProxyClient::new(provider);

        let quote_request = QuoteRequest {
            from_address: request.wallet_address.clone(),
            from_asset: request.from_asset.to_string(),
            to_asset: request.to_asset.to_string(),
            from_value: request.value.clone(),
            referral_address: request.clone().options.fee.unwrap().ton.address.clone(),
            referral_bps: request.clone().options.fee.unwrap().ton.bps as usize,
            slippage_bps: request.clone().options.slippage.bps as usize,
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
        let client = ProxyClient::new(provider);
        let data = client.get_quote_data(&self.url, route_data).await?;

        Ok(SwapQuoteData {
            to: data.to,
            value: data.value,
            data: data.data,
            approval: None,
            gas_limit: None,
        })
    }
}
