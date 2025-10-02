use super::{
    error::SwapperError,
    models::{FetchQuoteData, Permit2ApprovalData, SwapperChainAsset, SwapperProviderType, SwapperQuote, SwapperQuoteRequest, SwapperSwapResult},
    remote_models::{SwapperProviderMode, SwapperQuoteData},
};
use async_trait::async_trait;
use std::fmt::Debug;

use primitives::{Chain, swap::SwapStatus};

#[async_trait]
pub trait Swapper: Send + Sync + Debug {
    fn provider(&self) -> &SwapperProviderType;
    fn supported_assets(&self) -> Vec<SwapperChainAsset>;
    async fn fetch_quote(&self, request: &SwapperQuoteRequest) -> Result<SwapperQuote, SwapperError>;
    async fn fetch_permit2_for_quote(&self, _quote: &SwapperQuote) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        Ok(None)
    }
    async fn fetch_quote_data(&self, quote: &SwapperQuote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError>;
    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str) -> Result<SwapperSwapResult, SwapperError> {
        if self.provider().mode() == SwapperProviderMode::OnChain {
            Ok(self.get_onchain_swap_status(chain, transaction_hash))
        } else {
            Err(SwapperError::NotImplemented)
        }
    }

    /// Default OnChain provider swap status implementation
    fn get_onchain_swap_status(&self, chain: Chain, transaction_hash: &str) -> SwapperSwapResult {
        SwapperSwapResult {
            status: SwapStatus::Completed,
            from_chain: chain,
            from_tx_hash: transaction_hash.to_string(),
            to_chain: Some(chain),
            to_tx_hash: Some(transaction_hash.to_string()),
        }
    }
}

impl dyn Swapper {
    pub fn supported_chains(&self) -> Vec<Chain> {
        self.supported_assets()
            .into_iter()
            .map(|x| match x.clone() {
                SwapperChainAsset::All(chain) => chain,
                SwapperChainAsset::Assets(chain, _) => chain,
            })
            .collect()
    }
}
