use crate::error::DeFiError;
use async_trait::async_trait;
use primitives::{Chain, DeFiPortfolio, DeFiPosition, DeFiPositionFilters};

#[async_trait]
pub trait DeFiProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &'static str;

    /// Get supported chains
    fn supported_chains(&self) -> Vec<Chain>;

    /// Get complete portfolio for an address
    async fn get_portfolio(&self, address: &str, chains: Vec<Chain>) -> Result<DeFiPortfolio, DeFiError>;

    /// Get positions for an address with optional filters
    async fn get_positions(&self, address: &str, filters: Option<DeFiPositionFilters>) -> Result<Vec<DeFiPosition>, DeFiError>;
}
