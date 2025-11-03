use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    alien::RpcProvider,
    asset::{HYPERCORE_HYPE, HYPEREVM_HYPE},
};
use gem_hypercore::is_spot_swap;

use super::{bridge::HyperCoreBridge, spot::HyperCoreSpot};

#[derive(Debug)]
pub struct Hyperliquid {
    provider: ProviderType,
    spot: HyperCoreSpot,
    bridge: HyperCoreBridge,
}

impl Hyperliquid {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Hyperliquid),
            spot: HyperCoreSpot::new(rpc_provider),
            bridge: HyperCoreBridge::new(),
        }
    }

    fn is_spot_request(request: &QuoteRequest) -> bool {
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();
        is_spot_swap(from_chain, to_chain)
    }

    fn is_bridge_request(request: &QuoteRequest) -> bool {
        let from_id = request.from_asset.asset_id();
        let to_id = request.to_asset.asset_id();

        (from_id == HYPERCORE_HYPE.id && to_id == HYPEREVM_HYPE.id) || (from_id == HYPEREVM_HYPE.id && to_id == HYPERCORE_HYPE.id)
    }

    fn is_bridge_quote(quote: &Quote) -> bool {
        Self::is_bridge_request(&quote.request)
    }
}

#[async_trait]
impl Swapper for Hyperliquid {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        let mut assets = self.spot.supported_assets();
        assets.extend(self.bridge.supported_assets());
        assets
    }

    async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        if Self::is_spot_request(request) {
            return self.spot.fetch_quote(request).await;
        }

        if Self::is_bridge_request(request) {
            return self.bridge.fetch_quote(request).await;
        }

        Err(SwapperError::NotSupportedPair)
    }

    async fn fetch_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        if Self::is_spot_request(&quote.request) {
            return self.spot.fetch_quote_data(quote, data).await;
        }

        if Self::is_bridge_quote(quote) {
            return self.bridge.fetch_quote_data(quote, data).await;
        }

        Err(SwapperError::NotSupportedPair)
    }

    async fn get_swap_result(&self, chain: primitives::Chain, transaction_hash: &str) -> Result<primitives::swap::SwapResult, SwapperError> {
        self.bridge.get_swap_result(chain, transaction_hash).await
    }
}
