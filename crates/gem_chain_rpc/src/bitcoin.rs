use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset, AssetBalance};

use gem_bitcoin::rpc::client::BitcoinClient;
use gem_bitcoin::rpc::mapper::BitcoinMapper;

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
        let status = self.client.get_status().await?;
        Ok(status.blockbook.best_height)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
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

        let transactions = transactions
            .into_iter()
            .flat_map(|x| BitcoinMapper::map_transaction(self.get_chain(), &x, block_number))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
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
