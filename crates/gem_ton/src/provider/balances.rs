use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use crate::provider::balances_mapper::{map_balance_tokens, map_coin_balance};
use crate::rpc::client::TonClient;

#[async_trait]
impl<C: Client> ChainBalances for TonClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_balance(address).await?;
        Ok(map_coin_balance(balance))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balances = self.get_jetton_wallets(address).await?;
        Ok(map_balance_tokens(balances, token_ids))
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}
