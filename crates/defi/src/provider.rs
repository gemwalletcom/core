use crate::error::DeFiError;
use async_trait::async_trait;
use primitives::{Chain, ChainType, DeFiPortfolio, DeFiPosition, DeFiPositionFilters};

#[async_trait]
pub trait DeFiProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn supported_chain_types(&self) -> Vec<ChainType>;

    /// Get complete portfolio for an address
    async fn get_portfolio(&self, address: &str, chains: Vec<Chain>) -> Result<DeFiPortfolio, DeFiError>;

    /// Get positions for an address with optional filters
    async fn get_positions(&self, address: &str, filters: Option<DeFiPositionFilters>) -> Result<Vec<DeFiPosition>, DeFiError>;
}
