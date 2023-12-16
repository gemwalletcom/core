use std::{error::Error, str::FromStr};

use crate::{sui::model::Digests, ChainProvider};
use async_trait::async_trait;
use chrono::Utc;
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use num_bigint::BigUint;
use primitives::{chain::Chain, Transaction, TransactionState, TransactionType};
use serde_json::json;

use super::model::GasUsed;

pub struct SuiClient {
    client: HttpClient,
}

impl SuiClient {
    pub fn new(url: String) -> Self {
        let client = HttpClientBuilder::default().build(url).unwrap();

        Self { client }
    }

    fn get_fee(&self, gas_used: GasUsed) -> BigUint {
        let computation_cost =
            BigUint::from_str(gas_used.computation_cost.as_str()).unwrap_or_default();
        let storage_cost = BigUint::from_str(gas_used.storage_cost.as_str()).unwrap_or_default();
        let storage_rebate =
            BigUint::from_str(gas_used.storage_rebate.as_str()).unwrap_or_default();
        let cost = computation_cost.clone() + storage_cost.clone();
        // fee is potentially negative for storage rebate, for now return 0
        if storage_rebate >= cost {
            return BigUint::from(0u32);
        }
        computation_cost + storage_cost - storage_rebate
    }

    fn map_transaction(
        &self,
        transaction: super::model::Digest,
        block_number: i64,
    ) -> Option<primitives::Transaction> {
        let balance_changes = transaction.balance_changes.unwrap_or_default();

        let chain = self.get_chain();

        // system transfer
        if balance_changes.len() == 2
            && balance_changes[0].coin_type == chain.as_denom()
            && balance_changes[1].coin_type == chain.as_denom()
        {
            let (from_change, to_change) = if balance_changes[0].amount.contains('-') {
                (balance_changes[0].clone(), balance_changes[1].clone())
            } else {
                (balance_changes[1].clone(), balance_changes[0].clone())
            };
            let from = from_change.owner.address_owner.unwrap_or_default();
            let to = to_change.owner.address_owner.unwrap_or_default();
            let fee = self.get_fee(transaction.effects.gas_used.clone());
            let value = to_change.amount;
            let state = if transaction.effects.status.status == "success" {
                TransactionState::Confirmed
            } else {
                TransactionState::Failed
            };

            let transaction = primitives::Transaction::new(
                transaction.digest.clone(),
                chain.as_asset_id(),
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                block_number.to_string(),
                0.to_string(),
                fee.to_string(),
                chain.as_asset_id(),
                value.to_string(),
                None,
                None,
                Utc::now(),
            );
            return Some(transaction);
        }
        None
    }
}

#[async_trait]
impl ChainProvider for SuiClient {
    fn get_chain(&self) -> Chain {
        Chain::Sui
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block: String = self
            .client
            .request("sui_getLatestCheckpointSequenceNumber", rpc_params![])
            .await?;
        Ok(block.parse::<i64>()?)
    }

    async fn get_transactions(
        &self,
        block_number: i64,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!({
                "filter": {
                    "Checkpoint": block_number.to_string()
                },
                "options": {
                    "showEffects": true,
                    "showInput": false,
                    "showBalanceChanges":  true
                }
            }),
            json!(null),
            json!(50),
            json!(true),
        ];

        let block: Digests = self
            .client
            .request("suix_queryTransactionBlocks", params)
            .await?;
        let transactions = block
            .data
            .into_iter()
            .flat_map(|x| self.map_transaction(x, block_number))
            .collect::<Vec<primitives::Transaction>>();
        return Ok(transactions);
    }
}
