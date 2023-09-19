use std::{error::Error, str::FromStr};

use crate::{ChainProvider, sui::model::Digests};
use async_trait::async_trait;
use chrono::Utc;
use ethers::providers::{JsonRpcClient, Http, RetryClientBuilder, RetryClient};
use num_bigint::BigUint;
use primitives::{chain::Chain, Transaction, TransactionType, TransactionState, TransactionDirection, asset_id::AssetId};
use reqwest::Url;
use serde_json::json;

use super::model::GasUsed;

pub struct SuiClient {
    client: RetryClient<Http>,
}

impl SuiClient {
    pub fn new(url: String) -> Self {
        let provider = Http::new(Url::parse(url.as_str()).unwrap());
        let client = RetryClientBuilder::default()
            .build(provider, Box::<ethers::providers::HttpRateLimitRetryPolicy>::default());
        
        Self {
            client,
        }
    }

    fn get_fee(&self, gas_used: GasUsed) -> BigUint {
        let computation_cost = BigUint::from_str(gas_used.computation_cost.as_str()).unwrap_or_default();
        let storage_cost = BigUint::from_str(gas_used.storage_cost.as_str()).unwrap_or_default();
        let storage_rebate = BigUint::from_str(gas_used.storage_rebate.as_str()).unwrap_or_default();
        return computation_cost + storage_cost - storage_rebate;
    }

    fn map_transaction(&self, transaction: super::model::Digest, block_number: i64) -> Option<primitives::Transaction> {
        let balance_changes = transaction.balance_changes.unwrap_or_default();
        
        let chain = self.get_chain();

        // system transfer
        if balance_changes.len() == 2 && balance_changes[0].coin_type == chain.as_denom() && balance_changes[1].coin_type == chain.as_denom()  {    
            
            let from = balance_changes[0].owner.address_owner.clone(); 
            let to = balance_changes[1].owner.address_owner.clone();
            let fee = self.get_fee(transaction.effects.gas_used.clone());
            let value = balance_changes[1].amount.clone();
            let state = if transaction.effects.status.status == "success" { TransactionState::Confirmed } else { TransactionState::Failed} ;        

            let transaction = primitives::Transaction{ 
                id: "".to_string(), 
                hash: transaction.digest.clone(),
                asset_id: AssetId::from_chain(chain), 
                from, 
                to, 
                contract: None, 
                transaction_type: TransactionType::Transfer, 
                state, 
                block_number: block_number as i32, 
                sequence: 0, 
                fee: fee.to_string(), 
                fee_asset_id: AssetId::from_chain(chain), 
                value: value.to_string(), 
                memo: None,
                direction: TransactionDirection::SelfTransfer, 
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            };
            return Some(transaction);
        }
        return None 
    }
}

#[async_trait]
impl ChainProvider for SuiClient {

    fn get_chain(&self) -> Chain {
        Chain::Sui
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block: String = JsonRpcClient::request(&self.client, "sui_getLatestCheckpointSequenceNumber", ()).await?;
        Ok(block.parse::<i64>().unwrap_or_default())
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "filter": {
                    "Checkpoint": block_number.to_string()
                },
                "options": {
                    "showEffects": true,
                    "showInput": false,
                    "showBalanceChanges":  true
                }
            },
            null,
            50,
            true
        ]);
        let block: Digests = JsonRpcClient::request(&self.client, "suix_queryTransactionBlocks", params).await?;
        let transactions = block.data.into_iter()
            .flat_map(|x| self.map_transaction(x, block_number))
            .collect::<Vec<primitives::Transaction>>();
        return Ok(transactions)
    }
}