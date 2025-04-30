use std::sync::Arc;

use crate::{
    network::AlienProvider,
    swapper::{
        asset::{ARBITRUM_USDC, ETHEREUM_USDC, ETHEREUM_USDT, SOLANA_USDC},
        FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProviderData, SwapProviderType, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, Swapper,
        SwapperError,
    },
};
use primitives::{swap::QuoteAsset, Chain};

use super::{broker::BrokerClient, capitalize::capitalize_first_letter, model::QuoteRequest};

#[derive(Debug)]
pub struct ChainflipProvider {
    provider: SwapProviderType,
}

impl Default for ChainflipProvider {
    fn default() -> Self {
        Self {
            provider: SwapProviderType::new(GemSwapProvider::Chainflip),
        }
    }
}

impl ChainflipProvider {
    fn map_asset_id(asset: &QuoteAsset) -> (String, String) {
        let asset_id = asset.asset_id();
        let chain_name = capitalize_first_letter(asset_id.chain.as_ref());
        (chain_name, asset.symbol.clone())
    }
}

#[async_trait::async_trait]
impl Swapper for ChainflipProvider {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        vec![
            SwapChainAsset::Assets(Chain::Bitcoin, vec![]),
            SwapChainAsset::Assets(Chain::Ethereum, vec![ETHEREUM_USDC.id.clone(), ETHEREUM_USDT.id.clone()]),
            SwapChainAsset::Assets(Chain::Solana, vec![SOLANA_USDC.id.clone()]),
            SwapChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_USDC.id.clone()]),
        ]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let broker_client = BrokerClient::new(provider);
        let (src_chain, src_asset) = Self::map_asset_id(&request.from_asset);
        let (dest_chain, dest_asset) = Self::map_asset_id(&request.to_asset);
        let quote_request = QuoteRequest {
            amount: request.value.clone(),
            src_chain,
            src_asset,
            dest_chain,
            dest_asset,
            is_vault_swap: true,
            dca_enabled: true,
        };

        let quote_response = broker_client.get_quote(&quote_request).await?;
        let quote = SwapQuote {
            from_value: request.value.clone(),
            to_value: quote_response.egress_amount.clone(),
            data: SwapProviderData {
                provider: self.provider.clone(),
                slippage_bps: quote_response.slippage_bps(),
                routes: vec![SwapRoute {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: "{}".to_string(),
                    gas_limit: None,
                }],
            },
            eta_in_seconds: Some(quote_response.estimated_duration_seconds as u32),
            request: request.clone(),
        };
        Ok(quote)
    }

    async fn fetch_quote_data(&self, _quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        Err(SwapperError::NotImplemented)
    }

    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        Err(SwapperError::NotImplemented)
    }
}
