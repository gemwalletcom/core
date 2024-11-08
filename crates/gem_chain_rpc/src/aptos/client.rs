use std::{error::Error, str::FromStr};

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use chrono::Utc;
use num_bigint::BigUint;
use primitives::{chain::Chain, Asset, TransactionState, TransactionType};
use reqwest_middleware::ClientWithMiddleware;

use super::model::{Block, Ledger, DEPOSIT_EVENT};

pub struct AptosClient {
    url: String,
    client: ClientWithMiddleware,
}

impl AptosClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub fn map_transaction(&self, transaction: super::model::Transaction, block_number: i64) -> Option<primitives::Transaction> {
        let events = transaction.clone().events.unwrap_or_default();

        if transaction.transaction_type == "user_transaction" && events.len() <= 4 {
            let deposit_event = events.iter().find(|x| x.event_type == DEPOSIT_EVENT)?;

            let asset_id = self.get_chain().as_asset_id();
            let state = if transaction.success {
                TransactionState::Confirmed
            } else {
                TransactionState::Failed
            };
            let to = &deposit_event.guid.account_address;
            let value = &deposit_event.get_amount()?;
            let gas_used = BigUint::from_str(transaction.gas_used.unwrap_or_default().as_str()).unwrap_or_default();
            let gas_unit_price = BigUint::from_str(transaction.gas_unit_price.unwrap_or_default().as_str()).unwrap_or_default();
            let fee = gas_used * gas_unit_price;

            let transaction = primitives::Transaction::new(
                transaction.hash,
                asset_id.clone(),
                transaction.sender.unwrap_or_default(),
                to.clone(),
                None,
                TransactionType::Transfer,
                state,
                block_number.to_string(),
                transaction.sequence_number.unwrap_or_default(),
                fee.to_string(),
                asset_id,
                value.clone(),
                None,
                None,
                Utc::now(),
            );
            return Some(transaction);
        }
        None
    }

    pub async fn get_ledger(&self) -> Result<Ledger, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v1/", self.url);
        let response = self.client.get(url).send().await?.json::<Ledger>().await?;
        Ok(response)
    }

    pub async fn get_block_transactions(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v1/blocks/by_height/{}?with_transactions=true", self.url, block_number);
        let response = self.client.get(url).send().await?.json::<Block>().await?;

        Ok(response)
    }
}

#[async_trait]
impl ChainBlockProvider for AptosClient {
    fn get_chain(&self) -> Chain {
        Chain::Aptos
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let ledger = self.get_ledger().await?;
        Ok(ledger.block_height.parse::<i64>().unwrap_or_default())
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.get_block_transactions(block_number).await?.transactions;
        let transactions = transactions
            .into_iter()
            .flat_map(|x| self.map_transaction(x, block_number))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for AptosClient {
    async fn get_token_data(&self, _chain: Chain, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
