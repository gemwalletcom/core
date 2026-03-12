use async_trait::async_trait;
use primitives::{AssetId, Chain, ContractCallData, DelegationBase, DelegationValidator, YieldProvider};

use crate::error::YielderError;

#[async_trait]
pub trait EarnProvider: Send + Sync {
    fn id(&self) -> YieldProvider;
    fn earn_providers(&self, asset_id: &AssetId) -> Vec<DelegationValidator>;
    fn earn_asset_ids_for_chain(&self, chain: Chain) -> Vec<AssetId>;

    async fn positions(&self, chain: Chain, address: &str, asset_ids: &[AssetId]) -> Result<Vec<DelegationBase>, YielderError>;
    async fn deposit(&self, asset_id: &AssetId, address: &str, value: &str) -> Result<ContractCallData, YielderError>;
    async fn withdraw(&self, asset_id: &AssetId, address: &str, value: &str, shares: &str) -> Result<ContractCallData, YielderError>;
}
