use std::error::Error;

use async_trait::async_trait;
use primitives::{AssetBalance, DelegationBase, DelegationValidator, FeePriorityValue, TransactionUpdate, TransactionStateRequest, UTXO};

pub trait ChainTraits: ChainBalances + ChainStaking + ChainTransactions + ChainState + ChainAccount {}

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
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>>;
}

#[async_trait]
pub trait ChainState: Send + Sync {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>>;
    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>>;
    async fn get_fees(&self) -> Result<Vec<FeePriorityValue>, Box<dyn Error + Sync + Send>>;
}

#[async_trait]
pub trait ChainAccount: Send + Sync {
    async fn get_utxos(&self, address: String) -> Result<Vec<UTXO>, Box<dyn Error + Sync + Send>>;
}
