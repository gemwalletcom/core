use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::Transaction;
use primitives::{chain::Chain, Asset, AssetBalance};

use futures::future::try_join_all;
use gem_bitcoin::rpc::BitcoinClient;
use gem_bitcoin::rpc::BitcoinMapper;

pub struct BitcoinProvider {
    client: BitcoinClient,
}

impl BitcoinProvider {
    pub fn new(client: BitcoinClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for BitcoinProvider {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_status().await?.blockbook.best_height)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let mut page: usize = 1;
        let limit: usize = 20;
        let mut transactions = Vec::new();
        loop {
            let block = self.client.get_block(block_number, page, limit).await?;
            transactions.extend(block.txs.clone());
            if block.page == block.total_pages {
                break;
            }
            page += 1;
        }
        Ok(BitcoinMapper::map_transactions(self.get_chain(), transactions))
    }
}

#[async_trait]
impl ChainTokenDataProvider for BitcoinProvider {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}

#[async_trait]
impl ChainAssetsProvider for BitcoinProvider {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl ChainTransactionsProvider for BitcoinProvider {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let address = self.client.get_address(&address).await?;
        let transaction_ids = address.txids.iter().take(6).collect::<Vec<_>>();
        let transactions = try_join_all(transaction_ids.into_iter().map(|x| self.client.get_transaction(x)))
            .await?
            .into_iter()
            .collect::<Vec<_>>();
        Ok(BitcoinMapper::map_transactions(self.get_chain(), transactions))
    }
}
