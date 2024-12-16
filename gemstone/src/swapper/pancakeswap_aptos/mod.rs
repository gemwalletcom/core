use std::sync::Arc;

mod client;
mod model;
use super::{
    ApprovalType, FetchQuoteData, GemSwapProvider, SwapProvider, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError,
};

use crate::network::AlienProvider;
use async_trait::async_trait;
use client::PancakeSwapAptosClient;
use gem_aptos::model::{TransactionPayload, NATIVE_APTOS_COIN};
use model::{RouteData, PANCAKE_SWAP_APTOS_ADDRESS};
use primitives::{AssetId, Chain};

#[derive(Debug, Default)]
pub struct PancakeSwapAptos {}

impl PancakeSwapAptos {
    fn to_asset(&self, asset_id: AssetId) -> String {
        if let Some(token_id) = asset_id.token_id {
            return token_id;
        }
        NATIVE_APTOS_COIN.to_string()
    }

    fn swap_exact_input(&self, assets: Vec<String>, from_value: String, to_value: String) -> TransactionPayload {
        let function = match assets.len() {
            2 => "swap_exact_input",
            3 => "swap_exact_input_doublehop",
            4 => "swap_exact_input_triplehop",
            _ => unimplemented!(),
        };

        TransactionPayload {
            function: function.to_string(),
            type_arguments: assets,
            arguments: vec![from_value, to_value],
            payload_type: "entry_function_payload".to_string(),
        }
    }
}

#[async_trait]
impl GemSwapProvider for PancakeSwapAptos {
    fn provider(&self) -> SwapProvider {
        SwapProvider::PancakeSwapAptosV2
    }

    fn supported_chains(&self) -> Vec<Chain> {
        vec![Chain::Aptos]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let endpoint: String = provider.get_endpoint(Chain::Aptos).unwrap();
        let client = PancakeSwapAptosClient::new(provider);

        let from_internal_asset = self.to_asset(request.from_asset.clone());
        let to_internal_asset = self.to_asset(request.to_asset.clone());

        let quote_value = client
            .get_quote(
                endpoint.as_str(),
                from_internal_asset.as_str(),
                to_internal_asset.as_str(),
                request.value.to_string().as_str(),
                request.options.slippage_bps,
            )
            .await?;

        let route_data = RouteData {
            min_value: quote_value.clone(),
            assets: vec![from_internal_asset, to_internal_asset],
        };
        let route_data = serde_json::to_string(&route_data).unwrap();

        let quote = SwapQuote {
            from_value: request.value.clone(),
            to_value: quote_value.clone(),
            data: SwapProviderData {
                provider: self.provider(),
                suggested_slippage_bps: None,
                routes: vec![SwapRoute {
                    input: request.from_asset.clone(),
                    output: request.to_asset.clone(),
                    route_data,
                    gas_estimate: None,
                }],
            },
            approval: ApprovalType::None,
            request: request.clone(),
        };

        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let routes = quote.data.clone().routes;
        let route_data: RouteData = serde_json::from_str(&routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;

        let payload = self.swap_exact_input(
            vec![self.to_asset(quote.request.from_asset.clone()), self.to_asset(quote.request.to_asset.clone())],
            quote.from_value.clone().to_string(),
            route_data.min_value.clone(),
        );

        let data = SwapQuoteData {
            to: PANCAKE_SWAP_APTOS_ADDRESS.to_string(),
            value: quote.from_value.clone(),
            data: serde_json::to_string(&payload).unwrap(),
        };
        Ok(data)
    }

    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        unimplemented!()
    }
}
