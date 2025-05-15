use std::error::Error;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use chrono::Utc;
use primitives::{chain::Chain, transaction_utxo::TransactionInput, Asset, TransactionDirection, TransactionType};

use super::client::BitcoinClient;
use super::model::Transaction;

pub struct BitcoinProvider {
    client: BitcoinClient,
}

impl BitcoinProvider {
    pub fn new(client: BitcoinClient) -> Self {
        Self { client }
    }

    pub fn map_transaction(chain: Chain, transaction: &Transaction, _block_number: i64) -> Option<primitives::Transaction> {
        let inputs: Vec<TransactionInput> = transaction
            .vin
            .iter()
            .filter(|i| i.is_address)
            .map(|input| TransactionInput {
                address: input.addresses.clone().unwrap().first().unwrap().to_string(),
                value: input.value.clone(),
            })
            .collect();

        let outputs: Vec<TransactionInput> = transaction
            .vout
            .iter()
            .filter(|o| o.is_address)
            .map(|output| TransactionInput {
                address: output.addresses.clone().unwrap_or_default().first().unwrap().to_string(),
                value: output.value.clone(),
            })
            .collect();

        if inputs.is_empty() || outputs.is_empty() {
            return None;
        }

        let transaction = primitives::Transaction::new_with_utxo(
            transaction.txid.clone(),
            chain.as_asset_id(),
            None,
            None,
            None,
            TransactionType::Transfer,
            primitives::TransactionState::Confirmed,
            transaction.block_height.to_string(),
            0.to_string(),
            transaction.fees.clone(),
            chain.as_asset_id(),
            "0".to_string(),
            None,
            TransactionDirection::SelfTransfer,
            inputs.into(),
            outputs.into(),
            None,
            Utc::now(),
            //Utc.timestamp_opt(transaction.block_time, 0).unwrap(),
        );

        Some(transaction)
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
            .flat_map(|x| Self::map_transaction(self.get_chain(), &x, block_number))
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
