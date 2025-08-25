use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainBalances;
#[cfg(feature = "rpc")]
use gem_client::Client;
use primitives::AssetBalance;

use crate::provider::balances_mapper;
use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainBalances for SuiClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_balance(address).await?;
        let asset_balance = balances_mapper::map_coin_balance(balance);
        Ok(asset_balance)
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let all_balances = self.get_all_balances(address).await?;
        let asset_balances = balances_mapper::map_token_balances(all_balances, token_ids);
        Ok(asset_balances)
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let delegations = self.get_stake_delegations(address).await?;
        let system_state = self.get_system_state().await?;
        let staking_balance = balances_mapper::map_staking_balance_with_system_state(delegations, system_state);
        Ok(staking_balance)
    }

    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let all_balances = self.get_all_balances(address).await?;
        let asset_balances = balances_mapper::map_assets_balances(all_balances);
        Ok(asset_balances)
    }
}
