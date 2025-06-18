use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::Transaction;
use primitives::{Asset, AssetBalance, Chain};

use gem_cosmos::rpc::CosmosClient;
use gem_cosmos::rpc::CosmosMapper;

pub struct CosmosProvider {
    client: CosmosClient,
}

impl CosmosProvider {
    pub fn new(client: CosmosClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for CosmosProvider {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block("latest").await?;
        Ok(block.block.header.height.parse::<i64>()?)
    }

    async fn get_transactions(&self, block: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let response = self.client.get_block(block.to_string().as_str()).await?;
        let transactions = response.block.data.txs;

        let txs = transactions
            .clone()
            .into_iter()
            .flat_map(|x| self.client.map_transaction_decode(x))
            .collect::<Vec<_>>();
        let txs_futures = txs.clone().into_iter().map(|x| self.client.get_transaction(x.hash));
        let receipts = futures::future::try_join_all(txs_futures).await?;

        let transactions = txs
            .clone()
            .into_iter()
            .zip(receipts.iter())
            .filter_map(|(transaction, receipt)| CosmosMapper::map_transaction(self.get_chain(), transaction, receipt.clone()))
            .collect::<Vec<Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for CosmosProvider {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}

#[async_trait]
impl ChainAssetsProvider for CosmosProvider {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![]) //TODO: ChainAssetsProvider
    }
}

#[async_trait]
impl ChainTransactionsProvider for CosmosProvider {
    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![]) //TODO: ChainTransactionsProvider
    }
}
