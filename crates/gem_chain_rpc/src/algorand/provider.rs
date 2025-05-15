use std::error::Error;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use chrono::DateTime;
use primitives::{Asset, Chain};

use super::client::AlgorandClient;
use super::model::{Block, Transaction, TRANSACTION_TYPE_PAY};

pub struct AlgorandProvider {
    client: AlgorandClient,
}

impl AlgorandProvider {
    pub fn new(client: AlgorandClient) -> Self {
        Self { client }
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
                DateTime::from_timestamp(block.ts, 0)?,
            )),
            _ => None,
        }
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

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let (block, transactions_ids) = self.client.get_block_transactions(block_number).await?;
        let transactions = block.clone().txns.unwrap_or_default();

        let transactions = transactions
            .iter()
            .zip(transactions_ids.iter())
            .flat_map(|(transaction, hash)| self.map_transaction(hash.clone(), block.clone(), transaction.txn.clone()))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for AlgorandProvider {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
