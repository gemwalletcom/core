use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use crate::{rpc::client::AptosClient, APTOS_NATIVE_COIN};
use super::balances_mapper::{map_token_balances, map_native_balance};

#[async_trait]
impl<C: Client> ChainBalances for AptosClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_account_balance(&address, APTOS_NATIVE_COIN).await?;
        Ok(map_native_balance(&balance.to_string(), self.get_chain()))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let resources = self.get_account_resources(&address).await?;
        Ok(map_token_balances(&resources, token_ids, self.get_chain()))
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}