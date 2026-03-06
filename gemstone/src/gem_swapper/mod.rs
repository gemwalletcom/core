mod error;
mod permit2;
use error::SwapperError;
use permit2::*;
mod remote_types;
use remote_types::*;
type Swapper = swapper::swapper::GemSwapper;

use crate::alien::{AlienProvider, AlienProviderWrapper};
use primitives::{AssetId, Chain};
use std::sync::Arc;

#[derive(Debug, uniffi::Object)]
pub struct GemSwapper {
    inner: Swapper,
}

#[uniffi::export]
impl GemSwapper {
    #[uniffi::constructor]
    pub fn new(rpc_provider: Arc<dyn AlienProvider>) -> Self {
        let wrapper = AlienProviderWrapper { provider: rpc_provider };
        Self {
            inner: Swapper::new(Arc::new(wrapper)),
        }
    }

    pub fn supported_chains(&self) -> Vec<Chain> {
        self.inner.supported_chains()
    }

    pub fn supported_chains_for_from_asset(&self, asset_id: &AssetId) -> SwapperAssetList {
        self.inner.supported_chains_for_from_asset(asset_id)
    }

    pub fn get_providers(&self) -> Vec<SwapperProviderType> {
        self.inner.get_providers()
    }

    pub fn get_providers_for_request(&self, request: &SwapperQuoteRequest) -> Result<Vec<SwapperProviderType>, SwapperError> {
        self.inner.get_providers_for_request(request)
    }

    pub async fn get_quote(&self, request: &SwapperQuoteRequest) -> Result<Vec<SwapperQuote>, SwapperError> {
        self.inner.get_quote(request).await
    }

    pub async fn fetch_quote_by_provider(&self, provider: SwapperProvider, request: SwapperQuoteRequest) -> Result<SwapperQuote, SwapperError> {
        self.inner.get_quote_by_provider(provider, request).await
    }

    pub async fn get_permit2_for_quote(&self, quote: &SwapperQuote) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        self.inner.get_permit2_for_quote(quote).await
    }

    pub async fn get_quote_data(&self, quote: &SwapperQuote, data: FetchQuoteData) -> Result<GemSwapQuoteData, SwapperError> {
        self.inner.get_quote_data(quote, data).await
    }

    pub async fn get_swap_result(&self, chain: Chain, provider: SwapperProvider, transaction_hash: &str) -> Result<SwapperSwapResult, SwapperError> {
        self.inner.get_swap_result(chain, provider, transaction_hash).await
    }
}
