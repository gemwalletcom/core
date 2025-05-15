use std::error::Error;

use primitives::{chain::Chain, Asset};

use super::model::{Block, BlockTransactions, BlockTransactionsInfo};
use reqwest_middleware::ClientWithMiddleware;

pub struct TronClient {
    url: String,
    client: ClientWithMiddleware,
}

impl TronClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub async fn get_block(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/wallet/getblock", self.url);
        let response = self.client.get(url).send().await?.json::<Block>().await?;
        Ok(response)
    }

    pub async fn get_block_tranactions(&self, block: i64) -> Result<BlockTransactions, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/walletsolidity/getblockbynum?num={}", self.url, block);
        let response = self.client.get(&url).send().await?.json::<BlockTransactions>().await?;
        Ok(response)
    }

    pub async fn get_block_tranactions_reciepts(&self, block: i64) -> Result<BlockTransactionsInfo, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/walletsolidity/gettransactioninfobyblocknum?num={}", self.url, block);
        let response = self.client.get(&url).send().await?.json::<BlockTransactionsInfo>().await?;
        Ok(response)
    }

    // Transaction mapping methods moved to TronMapper
}

impl TronClient {
    pub fn get_chain(&self) -> Chain {
        Chain::Tron
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block = self.get_block().await?;
        Ok(block.block_header.raw_data.number)
    }
    
    pub async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}

