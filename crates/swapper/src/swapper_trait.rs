use super::{
    SwapperProviderMode, SwapperQuoteData,
    cross_chain::VaultAddresses,
    error::SwapperError,
    models::{FetchQuoteData, Permit2ApprovalData, ProviderType, Quote, QuoteRequest, SwapResult, SwapperChainAsset},
};
use async_trait::async_trait;
use std::fmt::Debug;

use primitives::{Chain, swap::SwapStatus};

#[async_trait]
pub trait Swapper: Send + Sync + Debug {
    fn provider(&self) -> &ProviderType;
    fn supported_assets(&self) -> Vec<SwapperChainAsset>;
    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError>;
    async fn get_permit2_for_quote(&self, _quote: &Quote) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        Ok(None)
    }
    async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError>;
    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        Ok(VaultAddresses { deposit: vec![], send: vec![] })
    }
    async fn get_swap_result(&self, _chain: Chain, _transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        if self.provider().mode == SwapperProviderMode::OnChain {
            Ok(SwapResult {
                status: SwapStatus::Completed,
                metadata: None,
            })
        } else {
            Err(SwapperError::NotSupportedAsset)
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
