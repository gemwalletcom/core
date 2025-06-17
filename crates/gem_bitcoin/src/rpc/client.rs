use std::error::Error;

use crate::rpc::model::{AddressTransactions, Transaction};

use super::model::{Block, Status};
use primitives::chain::Chain;
use reqwest_middleware::ClientWithMiddleware;

pub struct BitcoinClient {
    chain: Chain,
    client: ClientWithMiddleware,
    url: String,
}

impl BitcoinClient {
    pub fn new(chain: Chain, client: ClientWithMiddleware, url: String) -> Self {
        Self { chain, client, url }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_status(&self) -> Result<Status, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/", self.url);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_block(&self, block_number: i64, page: usize, limit: usize) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/block/{}?page={}&limit={}", self.url, block_number, page, limit);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_address_transactions(&self, address: &str, limit: usize) -> Result<AddressTransactions, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/address/{}?pageSize={}&details=txs", self.url, address, limit);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_transaction(&self, txid: &str) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/tx/{}", self.url, txid);
        Ok(self.client.get(url).send().await?.json().await?)
    }
}
