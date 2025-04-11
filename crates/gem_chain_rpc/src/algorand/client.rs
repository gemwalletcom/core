use std::error::Error;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;

use chrono::DateTime;
use primitives::{Asset, Chain};
use reqwest_middleware::ClientWithMiddleware;

use super::model::{Block, BlockResponse, BlockTransactionIds, Transaction, TransactionsParams, TRANSACTION_TYPE_PAY};

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
        Ok(self.client.get(url).send().await?.json::<TransactionsParams>().await?)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/blocks/{}", self.url, block_number);
        Ok(self.client.get(url).send().await?.json::<BlockResponse>().await?.block)
    }

    pub async fn get_block_txids(&self, block_number: i64) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/blocks/{}/txids", self.url, block_number);
        Ok(self.client.get(url).send().await?.json::<BlockTransactionIds>().await?.block_txids)
    }

    pub fn map_transaction(&self, hash: String, block: Block, transaction: Transaction) -> Option<primitives::Transaction> {
        match transaction.transaction_type.as_str() {
            TRANSACTION_TYPE_PAY => Some(primitives::Transaction::new(
                hash,
                self.get_chain().as_asset_id(),
                transaction.clone().snd.unwrap_or_default(),
                transaction.clone().rcv.unwrap_or_default(),
                None,
                primitives::TransactionType::Transfer,
                primitives::TransactionState::Confirmed,
                block.rnd.to_string(),
                0.to_string(),
                transaction.fee.unwrap_or_default().to_string(),
                self.get_chain().as_asset_id(),
                transaction.amt.unwrap_or_default().to_string(),
                transaction.clone().get_memo(),
                None,
                DateTime::from_timestamp(block.ts, 0)?.naive_utc(),
            )),
            _ => None,
        }
    }
}

#[async_trait]
impl ChainBlockProvider for AlgorandClient {
    fn get_chain(&self) -> Chain {
        Chain::Algorand
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_transactions_params().await?.last_round)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.get_block(block_number).await?;
        let transactions = block.clone().txns.unwrap_or_default();
        let transactions_ids = self.get_block_txids(block_number).await?;

        let transactions = transactions
            .iter()
            .zip(transactions_ids.iter())
            .flat_map(|(transaction, hash)| self.map_transaction(hash.clone(), block.clone(), transaction.txn.clone()))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for AlgorandClient {
    async fn get_token_data(&self, _chain: Chain, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
