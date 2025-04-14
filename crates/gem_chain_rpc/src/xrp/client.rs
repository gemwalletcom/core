use std::error::Error;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use chrono::DateTime;
use primitives::{chain::Chain, Asset, AssetId, TransactionState, TransactionType};
use reqwest_middleware::ClientWithMiddleware;
use serde_json::json;

use super::model::{Ledger, LedgerCurrent, LedgerData, LedgerResult};

pub struct XRPClient {
    url: String,
    client: ClientWithMiddleware,
}

const RESULT_SUCCESS: &str = "tesSUCCESS";
const TRANSACTION_TYPE_PAYMENT: &str = "Payment";

impl XRPClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub fn map_transaction(&self, transaction: super::model::Transaction, block_number: i64, block_timestamp: i64) -> Option<primitives::Transaction> {
        if transaction.transaction_type == TRANSACTION_TYPE_PAYMENT && transaction.memos.unwrap_or_default().is_empty() {
            let memo = transaction.destination_tag.map(|x| x.to_string());
            let value = transaction.amount.clone()?.as_value_string()?;
            let token_id = transaction.amount?.token_id();
            let asset_id = AssetId::from(self.get_chain(), token_id);

            let state = if transaction.meta.result == RESULT_SUCCESS {
                TransactionState::Confirmed
            } else {
                TransactionState::Failed
            };
            // add check for delivered amount, for success it should be equal to amount
            let transaction = primitives::Transaction::new(
                transaction.hash,
                asset_id.clone(),
                transaction.account.unwrap_or_default(),
                transaction.destination.unwrap_or_default(),
                None,
                TransactionType::Transfer,
                state,
                block_number.to_string(),
                transaction.sequence.to_string(),
                transaction.fee.unwrap_or_default(),
                self.get_chain().as_asset_id(),
                value,
                memo,
                None,
                DateTime::from_timestamp(block_timestamp, 0)?,
            );
            return Some(transaction);
        }
        None
    }

    pub async fn get_ledger_current(&self) -> Result<LedgerCurrent, Box<dyn Error + Send + Sync>> {
        let params = json!(
            {
                "method": "ledger_current",
                "params": [{}]
            }
        );
        let response = self
            .client
            .post(self.url.clone())
            .json(&params)
            .send()
            .await?
            .json::<LedgerResult<LedgerCurrent>>()
            .await?;

        Ok(response.result)
    }

    pub async fn get_block_transactions(&self, block_number: i64) -> Result<Ledger, Box<dyn Error + Send + Sync>> {
        let params = json!(
            {
                "method": "ledger",
                "params": [
                    {
                        "ledger_index": block_number,
                        "transactions": true,
                        "expand": true
                    }
                ]
            }
        );
        let response = self
            .client
            .post(self.url.clone())
            .json(&params)
            .send()
            .await?
            .json::<LedgerResult<LedgerData>>()
            .await?;

        Ok(response.result.ledger)
    }
}

#[async_trait]
impl ChainBlockProvider for XRPClient {
    fn get_chain(&self) -> Chain {
        Chain::Xrp
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let ledger = self.get_ledger_current().await?;
        Ok(ledger.ledger_current_index)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.get_block_transactions(block_number).await?;
        let block_timestamp = 946684800 + block.close_time;
        let transactions = block.transactions;

        let transactions = transactions
            .into_iter()
            .flat_map(|x| self.map_transaction(x, block_number, block_timestamp))
            .collect::<Vec<primitives::Transaction>>();
        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for XRPClient {
    async fn get_token_data(&self, _chain: Chain, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
