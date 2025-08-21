use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use crate::provider::balances_mapper;
use crate::rpc::client::SolanaClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainBalances for SolanaClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_balance(&address).await?;
        Ok(balances_mapper::map_coin_balance(&balance))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let mut results = Vec::new();
        for token_id in token_ids {
            let accounts = self.get_token_accounts_by_mint(&address, &token_id).await?;
            results.extend(balances_mapper::map_token_accounts(&accounts, &token_id));
        }
        Ok(results)
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let accounts = self.get_staking_balance(&address).await?;
        Ok(balances_mapper::map_staking_balance(accounts))
    }
}
