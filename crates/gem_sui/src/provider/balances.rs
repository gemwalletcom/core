use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use crate::provider::balances_mapper;
use crate::rpc::client::SuiClient;

#[async_trait]
impl<C: Client + Clone> ChainBalances for SuiClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_balance(address).await?;
        Ok(balances_mapper::map_coin_balance(balance))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balances = self.get_all_balances(address).await?;
        Ok(balances_mapper::map_token_balances(balances, token_ids))
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let delegations = self.get_stake_delegations(address).await?;
        Ok(Some(balances_mapper::map_staking_balance(delegations)))
    }
}
