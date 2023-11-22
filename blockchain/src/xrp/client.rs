use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use chrono::Utc;
use primitives::{chain::Chain, TransactionType, TransactionState};
use reqwest_middleware::ClientWithMiddleware;
use serde_json::json;

use super::model::{LedgerCurrent, LedgerResult, Ledger, LedgerData};

pub struct XRPClient {
    url: String,
    client: ClientWithMiddleware,
}

impl XRPClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self {
            url,
            client,
        }
    }

    pub fn map_transaction(&self, transaction: super::model::Transaction, block_number: i64) -> Option<primitives::Transaction> {
        if transaction.transaction_type == "Payment" {
            let amount = transaction.amount.unwrap();
            match amount {
                // system transfer
                super::model::Amount::Str(value) => {
                    let asset_id = self.get_chain().as_asset_id();
                    let state = if transaction.meta.result == "tesSUCCESS" { TransactionState::Confirmed } else { TransactionState::Failed} ;
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
                        asset_id,
                        value,
                        Some(transaction.destination_tag.unwrap_or_default().to_string()),
                        None,
                        Utc::now()
                    );
                    return Some(transaction)
                },
                // token transfer
                super::model::Amount::Amount(_) => {
                    return None;
                },
            }
            
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
        let response = self.client
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
        let response = self.client
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
impl ChainProvider for XRPClient {

    fn get_chain(&self) -> Chain {
        Chain::Ripple
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let ledger = self.get_ledger_current().await?;
        Ok(ledger.ledger_current_index)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.get_block_transactions(block_number).await?.transactions;
        let transactions = transactions.into_iter()
            .flat_map(|x| self.map_transaction(x, block_number))
            .collect::<Vec<primitives::Transaction>>(); 
        Ok(transactions)
    }
}