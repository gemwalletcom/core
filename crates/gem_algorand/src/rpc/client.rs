use std::error::Error;

use reqwest_middleware::ClientWithMiddleware;

use crate::rpc::model::Account;

use super::model::{Block, BlockResponse, BlockTransactionIds, TransactionsParams};

pub struct AlgorandClient {
    url: String,
    client: ClientWithMiddleware,
}

impl AlgorandClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub async fn get_transactions_params(&self) -> Result<TransactionsParams, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/transactions/params", self.url);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_account(&self, address: &str) -> Result<Account, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/accounts/{}", self.url, address);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/blocks/{}", self.url, block_number);
        Ok(self.client.get(url).send().await?.json::<BlockResponse>().await?.block)
    }

    pub async fn get_block_txids(&self, block_number: i64) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/blocks/{}/txids", self.url, block_number);
        Ok(self.client.get(url).send().await?.json::<BlockTransactionIds>().await?.block_txids)
    }

    // Method to get transactions for a specific block (API calls only)
    pub async fn get_block_transactions(&self, block_number: i64) -> Result<(Block, Vec<String>), Box<dyn Error + Send + Sync>> {
        let block = self.get_block(block_number).await?;
        let transactions_ids = self.get_block_txids(block_number).await?;
        Ok((block, transactions_ids))
    }
}
