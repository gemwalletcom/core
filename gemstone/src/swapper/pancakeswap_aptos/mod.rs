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
use model::PANCAKE_SWAP_APTOS_ADDRESS;
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

        let quote_value = client
            .get_quote(
                endpoint.as_str(),
                self.to_asset(request.from_asset.clone()).as_str(),
                self.to_asset(request.to_asset.clone()).as_str(),
                request.value.to_string().as_str(),
                request.options.slippage_bps,
            )
            .await?;

        let quote = SwapQuote {
            from_value: request.value.clone(),
            to_value: quote_value.clone(),
            data: SwapProviderData {
                provider: self.provider(),
                suggested_slippage_bps: Some(0),
                routes: vec![SwapRoute {
                    input: request.from_asset.clone(),
                    output: request.to_asset.clone(),
                    route_data: quote_value,
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
        let route_data = routes.first().unwrap();

        let payload = TransactionPayload {
            function: format!("{}::router::swap_exact_input", PANCAKE_SWAP_APTOS_ADDRESS),
            type_arguments: vec![self.to_asset(quote.request.from_asset.clone()), self.to_asset(quote.request.to_asset.clone())],
            arguments: vec![quote.from_value.clone().to_string(), route_data.route_data.clone()],
            payload_type: "entry_function_payload".to_string(),
        };
        let data = serde_json::to_string(&payload).unwrap();
        let data = SwapQuoteData {
            to: PANCAKE_SWAP_APTOS_ADDRESS.to_string(),
            value: quote.from_value.clone(),
            data,
        };
        Ok(data)
    }

    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        unimplemented!()
    }
}
