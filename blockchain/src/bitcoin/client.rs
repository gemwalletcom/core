use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use primitives::{
    chain::Chain, transaction_utxo::TransactionInput, TransactionDirection, TransactionType,
};

use super::model::{Block, Status};
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

    pub async fn get_status(&self) -> Result<Status, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api", self.url);
        let response = self.client.get(url).send().await?.json::<Status>().await?;
        Ok(response)
    }

    pub async fn get_block(
        &self,
        block_number: i64,
    ) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/block/{}", self.url, block_number);
        let response = self.client.get(url).send().await?.json::<Block>().await?;
        Ok(response)
    }

    pub fn map_transaction(
        &self,
        transaction: super::model::Transaction,
        _block_number: i64,
    ) -> Option<primitives::Transaction> {
        let inputs: Vec<TransactionInput> = transaction
            .vin
            .iter()
            .filter(|i| i.is_address == true)
            .map(|input| TransactionInput {
                addresses: input.addresses.clone().unwrap(),
                value: input.value.clone(),
            })
            .collect();

        let outputs: Vec<TransactionInput> = transaction
            .vout
            .iter()
            .filter(|o| o.is_address == true)
            .map(|output| TransactionInput {
                addresses: output.addresses.clone(),
                value: output.value.clone(),
            })
            .collect();

        if inputs.is_empty() || outputs.is_empty() {
            return None;
        }

        let value = &outputs.last().unwrap().value;
        let from = inputs.first().unwrap().addresses.first().unwrap();
        let to = outputs.first().unwrap().addresses.first().unwrap();

        let transaction = primitives::Transaction::new_with_utxo(
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
            value.to_string(),
            None,
            TransactionDirection::SelfTransfer,
            inputs,
            outputs,
            Utc.timestamp_opt(transaction.block_time, 0).unwrap(),
        );

        Some(transaction)
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

    async fn get_transactions(
        &self,
        block_number: i64,
    ) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.get_block(block_number).await?.txs;
        let transactions = transactions
            .into_iter()
            .flat_map(|x| self.map_transaction(x, block_number))
            .collect::<Vec<primitives::Transaction>>();
        Ok(transactions)
    }
}
