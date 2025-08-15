use std::error::Error;

use async_trait::async_trait;
use primitives::AssetBalance;

#[async_trait]
pub trait ChainBalances: Send + Sync {
    async fn get_coin_balance(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>>;
    async fn get_tokens_balance(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>>;
    async fn get_stake_balance(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>>;
}
