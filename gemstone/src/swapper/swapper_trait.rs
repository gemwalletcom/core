use super::{
    error::SwapperError,
    models::{FetchQuoteData, Permit2ApprovalData, SwapperChainAsset, SwapperProviderType, SwapperQuote, SwapperQuoteRequest, SwapperSwapResult},
    remote_models::{SwapperProviderMode, SwapperQuoteData},
};
use crate::network::AlienProvider;
use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc};

use primitives::{block_explorer::get_block_explorers, swap::SwapStatus, Chain};

#[async_trait]
pub trait Swapper: Send + Sync + Debug {
    fn provider(&self) -> &SwapperProviderType;
    fn supported_assets(&self) -> Vec<SwapperChainAsset>;
    async fn fetch_quote(&self, request: &SwapperQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapperQuote, SwapperError>;
    async fn fetch_permit2_for_quote(&self, _quote: &SwapperQuote, _provider: Arc<dyn AlienProvider>) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        Ok(None)
    }
    async fn fetch_quote_data(&self, quote: &SwapperQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError>;
    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<SwapperSwapResult, SwapperError> {
        if self.provider().mode() == SwapperProviderMode::OnChain {
            Ok(self.get_onchain_swap_status(chain, transaction_hash))
        } else {
            Err(SwapperError::NotImplemented)
        }
    }

    /// Default OnChain provider swap status implementation
    fn get_onchain_swap_status(&self, chain: Chain, transaction_hash: &str) -> SwapperSwapResult {
        let explorers = get_block_explorers(chain);
        let explorer_url = if let Some(explorer) = explorers.first() {
            explorer.get_tx_url(transaction_hash)
        } else {
            String::new()
        };

        SwapperSwapResult {
            status: SwapStatus::Completed,
            from_chain: chain,
            from_tx_hash: transaction_hash.to_string(),
            to_chain: Some(chain),
            to_tx_hash: Some(transaction_hash.to_string()),
            explorer_url,
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
