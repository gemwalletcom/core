use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use super::balances_mapper::{map_native_balance, map_token_balances};
use crate::rpc::client::StellarClient;

#[async_trait]
impl<C: Client> ChainBalances for StellarClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = self.get_stellar_account(&address).await?;

        map_native_balance(&account, self.get_chain().as_asset_id(), self.get_chain())
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let account = self.get_stellar_account(&address).await?;
        Ok(map_token_balances(&account, token_ids, self.get_chain()))
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}
