use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::Transaction;
use primitives::{Asset, AssetBalance, Chain};

use gem_algorand::rpc::AlgorandClient;
use gem_algorand::rpc::AlgorandMapper;

pub struct AlgorandProvider {
    client: AlgorandClient,
}

impl AlgorandProvider {
    pub fn new(client: AlgorandClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for AlgorandProvider {
    fn get_chain(&self) -> Chain {
        Chain::Algorand
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_transactions_params().await?.last_round)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let (block, transactions_ids) = self.client.get_block_transactions(block_number).await?;
        let transactions = block.clone().txns.unwrap_or_default();

        let transactions = transactions
            .iter()
            .zip(transactions_ids.iter())
            .flat_map(|(transaction, hash)| AlgorandMapper::map_transaction(self.get_chain(), hash.clone(), block.clone(), transaction.txn.clone()))
            .collect::<Vec<Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for AlgorandProvider {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}

#[async_trait]
impl ChainAssetsProvider for AlgorandProvider {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let account = self.client.get_account(&address).await?;
        Ok(AlgorandMapper::map_assets_balance(account.assets))
    }
}

#[async_trait]
impl ChainTransactionsProvider for AlgorandProvider {
    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![]) //TODO: ChainTransactionsProvider
    }
}
