use std::{error::Error, str::FromStr};

use crate::sui::model::Digests;
use chrono::Utc;
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use num_bigint::BigUint;
use primitives::{chain::Chain, Asset, AssetId, AssetType, Transaction, TransactionState, TransactionType};
use serde_json::json;

use super::model::{CoinMetadata, EventStake, EventUnstake, GasUsed};

const SUI_STAKE_EVENT: &str = "0x3::validator::StakingRequestEvent";
const SUI_UNSTAKE_EVENT: &str = "0x3::validator::UnstakingRequestEvent";

pub struct SuiClient {
    client: HttpClient,
}

impl SuiClient {
    pub fn new(url: String) -> Self {
        let client = HttpClientBuilder::default().build(url).unwrap();

        Self { client }
    }

    fn get_fee(&self, gas_used: GasUsed) -> BigUint {
        let computation_cost = BigUint::from_str(gas_used.computation_cost.as_str()).unwrap_or_default();
        let storage_cost = BigUint::from_str(gas_used.storage_cost.as_str()).unwrap_or_default();
        let storage_rebate = BigUint::from_str(gas_used.storage_rebate.as_str()).unwrap_or_default();
        let cost = computation_cost.clone() + storage_cost.clone();
        // fee is potentially negative for storage rebate, for now return 0
        if storage_rebate >= cost {
            return BigUint::from(0u32);
        }
        computation_cost + storage_cost - storage_rebate
    }

    pub fn get_chain(&self) -> Chain {
        Chain::Sui
    }
    
    fn map_transaction(&self, transaction: super::model::Digest, block_number: i64) -> Option<primitives::Transaction> {
        let balance_changes = transaction.balance_changes.unwrap_or_default();
        let hash = transaction.digest.clone();
        let fee = self.get_fee(transaction.effects.gas_used.clone());
        let chain = self.get_chain();
        let created_at = Utc::now();
        // system transfer
        if balance_changes.len() == 2 && balance_changes[0].coin_type == chain.as_denom()? && balance_changes[1].coin_type == chain.as_denom()? {
            let (from_change, to_change) = if balance_changes[0].amount.contains('-') {
                (balance_changes[0].clone(), balance_changes[1].clone())
            } else {
                (balance_changes[1].clone(), balance_changes[0].clone())
            };

            let from = from_change.owner.get_address_owner();
            let to = to_change.owner.get_address_owner();

            if from.is_none() || to.is_none() {
                return None;
            }

            let value = to_change.amount;
            let state = if transaction.effects.status.status == "success" {
                TransactionState::Confirmed
            } else {
                TransactionState::Failed
            };

            let transaction = primitives::Transaction::new(
                hash,
                chain.as_asset_id(),
                from.unwrap_or_default(),
                to.unwrap_or_default(),
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
                created_at,
            );
            return Some(transaction);
        }
        // stake
        if transaction.events.len() == 1 && transaction.events.first()?.event_type == SUI_STAKE_EVENT {
            let event = transaction.events.first()?;
            let event_json = event.parsed_json.clone()?;
            let stake = serde_json::from_value::<EventStake>(event_json).ok()?;

            let transaction = primitives::Transaction::new(
                hash,
                chain.as_asset_id(),
                stake.staker_address,
                stake.validator_address,
                None,
                TransactionType::StakeDelegate,
                TransactionState::Confirmed,
                block_number.to_string(),
                0.to_string(),
                fee.to_string(),
                chain.as_asset_id(),
                stake.amount,
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }
        // unstake
        if transaction.events.len() == 1 && transaction.events.first()?.event_type == SUI_UNSTAKE_EVENT {
            let event = transaction.events.first()?;
            let event_json = event.parsed_json.clone()?;
            let stake: EventUnstake = serde_json::from_value::<EventUnstake>(event_json).ok()?;
            let value = stake.principal_amount; // add reward amount

            let transaction = primitives::Transaction::new(
                hash,
                chain.as_asset_id(),
                stake.staker_address,
                stake.validator_address,
                None,
                TransactionType::StakeUndelegate,
                TransactionState::Confirmed,
                block_number.to_string(),
                0.to_string(),
                fee.to_string(),
                chain.as_asset_id(),
                value,
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }

        None
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block: String = self.client.request("sui_getLatestCheckpointSequenceNumber", rpc_params![]).await?;
        Ok(block.parse::<i64>()?)
    }

    pub async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!({
                "filter": {
                    "Checkpoint": block_number.to_string()
                },
                "options": {
                    "showEffects": true,
                    "showInput": false,
                    "showBalanceChanges":  true,
                    "showEvents": true
                }
            }),
            json!(null),
            json!(50),
            json!(true),
        ];

        let block: Digests = self.client.request("suix_queryTransactionBlocks", params).await?;
        let transactions = block
            .data
            .into_iter()
            .flat_map(|x| self.map_transaction(x, block_number))
            .collect::<Vec<primitives::Transaction>>();
        return Ok(transactions);
    }
    
    pub async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let metadata: CoinMetadata = self.client.request("suix_getCoinMetadata", vec![token_id.clone()]).await?;

        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            metadata.name,
            metadata.symbol,
            metadata.decimals,
            AssetType::TOKEN,
        ))
    }
}
