use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::{AssetBalance, BitcoinChain};

use super::balances_mapper::map_balance_coin;
use crate::rpc::client::BitcoinClient;

impl<C: Client> BitcoinClient<C> {
    fn full_address(&self, address: &str) -> String {
        match self.chain {
            BitcoinChain::BitcoinCash => format!("bitcoincash:{}", address),
            _ => address.to_string(),
        }
    }
}

#[async_trait]
impl<C: Client> ChainBalances for BitcoinClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let full_address = self.full_address(&address);
        let account = self.get_balance(&full_address).await?;
        Ok(map_balance_coin(&account, self.chain))
    }

    async fn get_balance_tokens(&self, _address: String, _token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}
