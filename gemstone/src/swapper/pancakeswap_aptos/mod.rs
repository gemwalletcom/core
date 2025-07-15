use std::sync::Arc;

mod client;
mod model;
use super::{
    FetchQuoteData, SwapperProvider, SwapperChainAsset, SwapperProviderData, SwapperProviderType, SwapperQuote, SwapperQuoteData, SwapperQuoteRequest, SwapperRoute, Swapper,
    SwapperError,
};

use crate::network::AlienProvider;
use async_trait::async_trait;
use client::PancakeSwapAptosClient;
use gem_aptos::{TransactionPayload, APTOS_NATIVE_COIN};
use model::{RouteData, PANCAKE_SWAP_APTOS_ADDRESS};
use primitives::{AssetId, Chain};

#[derive(Debug)]
pub struct PancakeSwapAptos {
    pub provider: SwapperProviderType,
}

impl Default for PancakeSwapAptos {
    fn default() -> Self {
        Self {
            provider: SwapperProviderType::new(SwapperProvider::PancakeswapAptosV2),
        }
    }
}

impl PancakeSwapAptos {
    fn to_asset(&self, asset_id: AssetId) -> String {
        if let Some(token_id) = asset_id.token_id {
            return token_id;
        }
        APTOS_NATIVE_COIN.to_string()
    }

    fn router_swap_input(&self, address: &str, assets: Vec<String>, from_value: String, to_value: String) -> TransactionPayload {
        let function = match assets.len() {
            2 => "swap_exact_input",
            3 => "swap_exact_input_doublehop",
            4 => "swap_exact_input_triplehop",
            _ => unimplemented!(),
        };
        TransactionPayload {
            function: format!("{address}::router::{function}"),
            type_arguments: assets,
            arguments: vec![from_value, to_value],
            payload_type: "entry_function_payload".to_string(),
        }
    }
}

#[async_trait]
impl Swapper for PancakeSwapAptos {
    fn provider(&self) -> &SwapperProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Aptos)]
    }

    async fn fetch_quote(&self, request: &SwapperQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapperQuote, SwapperError> {
        let endpoint: String = provider.get_endpoint(Chain::Aptos).unwrap();
        let client = PancakeSwapAptosClient::new(provider);

        let from_internal_asset = self.to_asset(request.from_asset.asset_id());
        let to_internal_asset = self.to_asset(request.to_asset.asset_id());
        let fee_bps = 0; // TODO: implement fees

        let quote_value = client
            .get_quote(
                endpoint.as_str(),
                from_internal_asset.as_str(),
                to_internal_asset.as_str(),
                request.value.to_string().as_str(),
                request.options.slippage.bps + fee_bps,
            )
            .await?;

        let route_data = RouteData {
            min_value: quote_value.clone(),
            assets: vec![from_internal_asset, to_internal_asset],
        };
        let route_data = serde_json::to_string(&route_data).unwrap();

        let quote = SwapperQuote {
            from_value: request.value.clone(),
            to_value: quote_value.clone(),
            data: SwapperProviderData {
                provider: self.provider().clone(),
                routes: vec![SwapperRoute {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data,
                    gas_limit: None,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        };

        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &SwapperQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let routes = quote.data.clone().routes;
        let route_data: RouteData = serde_json::from_str(&routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;

        let payload = self.router_swap_input(
            PANCAKE_SWAP_APTOS_ADDRESS,
            vec![
                self.to_asset(quote.request.from_asset.asset_id()),
                self.to_asset(quote.request.to_asset.asset_id()),
            ],
            quote.from_value.clone().to_string(),
            route_data.min_value.clone(),
        );

        let data = SwapperQuoteData {
            to: PANCAKE_SWAP_APTOS_ADDRESS.to_string(),
            value: quote.from_value.clone(),
            data: serde_json::to_string(&payload).unwrap(),
            approval: None,
            gas_limit: None,
        };
        Ok(data)
    }
}
