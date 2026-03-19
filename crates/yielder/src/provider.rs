use async_trait::async_trait;
use primitives::{AssetBalance, AssetId, Chain, ContractCallData, DelegationBase, DelegationValidator, EarnType};

use crate::error::YielderError;

#[async_trait]
pub trait EarnProvider: Send + Sync {
    fn get_provider(&self, asset_id: &AssetId) -> Option<DelegationValidator>;

    async fn get_position(&self, address: &str, asset_id: &AssetId) -> Result<Option<DelegationBase>, YielderError>;
    async fn get_balance(&self, chain: Chain, address: &str, token_ids: &[String]) -> Result<Vec<AssetBalance>, YielderError>;
    async fn get_data(&self, asset_id: &AssetId, address: &str, value: &str, earn_type: &EarnType) -> Result<ContractCallData, YielderError>;
}
