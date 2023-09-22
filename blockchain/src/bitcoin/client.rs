use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use ethers::types::transaction;
use primitives::{chain::Chain, TransactionDirection, TransactionType};

use super::model::{Block, Status};
use reqwest_middleware::ClientWithMiddleware;

pub struct BitcoinClient {
    chain: Chain,
    client: ClientWithMiddleware,
    url: String,
}

impl BitcoinClient {
    pub fn new(chain: Chain, client: ClientWithMiddleware, url: String) -> Self {
        Self {
            chain,
            client,
            url,
        }
    }

    pub async fn get_status(&self) -> Result<Status, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api", self.url);
        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<Status>()
            .await?;
        Ok(response)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/block/{}", self.url, block_number);
        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<Block>()
            .await?;
        Ok(response)
    }

    pub fn map_transaction(&self, transaction: super::model::Transaction, _block_number: i64) -> Option<primitives::Transaction> {
        // only allow basic transfer support, from 1 adddress to another.
        if transaction.vin.first().unwrap().addresses.clone().unwrap_or_default().len() == 0 || 
            transaction.vout.len() > 2 {
            return None
        }
        let from_addresses = transaction.vin.first().unwrap().addresses.clone().unwrap_or_default();
        // destination usually is the last address, change is the first
        let to_addresses = transaction.vout.last().unwrap().addresses.clone().unwrap_or_default();

        let from = from_addresses.first().unwrap();
        let to = to_addresses.first().unwrap();

        let transaction = primitives::Transaction::new(
            transaction.txid,
            self.get_chain().as_asset_id(),
            from.to_string(),
            to.to_string(),
            None,
            TransactionType::Transfer,
            primitives::TransactionState::Confirmed,
            transaction.block_height.to_string(),
            0.to_string(),
            transaction.fees,
            self.get_chain().as_asset_id(),
            transaction.value,
            None,
            TransactionDirection::SelfTransfer,
            NaiveDateTime::from_timestamp_opt(transaction.block_time, 0).unwrap(),
        );

        return Some(transaction);
   }
}

#[async_trait]
impl ChainProvider for BitcoinClient {

    fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let status = self.get_status().await?;
        Ok(status.blockbook.best_height)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.get_block(block_number).await?.txs;
        let transactions = transactions.into_iter()
            .flat_map(|x| self.map_transaction(x, block_number))
            .collect::<Vec<primitives::Transaction>>();
        Ok(transactions)
    }
}