use std::error::Error;

use async_trait::async_trait;
use primitives::{AssetBalance, DelegationBase, DelegationValidator};

pub trait ChainTraits: ChainBalances + ChainStaking + ChainTransactions {}

#[async_trait]
pub trait ChainBalances: Send + Sync {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>>;
    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>>;
    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>>;
}

#[async_trait]
pub trait ChainStaking: Send + Sync {
    async fn get_staking_validators(&self) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>>;
    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>>;
}

#[async_trait]
pub trait ChainTransactions: Send + Sync {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>>;
    async fn get_transaction_status(&self, hash: String) -> Result<String, Box<dyn Error + Sync + Send>>;
}
