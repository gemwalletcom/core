use chrono::Utc;
use num_bigint::BigUint;
use primitives::{chain::Chain, Transaction, TransactionState, TransactionType};
use std::str::FromStr;

use super::{
    client::{SUI_STAKE_EVENT, SUI_UNSTAKE_EVENT},
    model::{Digest as SuiTransaction, GasUsed},
};

pub struct SuiMapper;

impl SuiMapper {
    pub fn get_fee(gas_used: GasUsed) -> BigUint {
        let computation_cost = BigUint::from_str(gas_used.computation_cost.as_str()).unwrap_or_default();
        let storage_cost = BigUint::from_str(gas_used.storage_cost.as_str()).unwrap_or_default();
        let storage_rebate = BigUint::from_str(gas_used.storage_rebate.as_str()).unwrap_or_default();

        let cost = computation_cost.clone() + storage_cost.clone();
        if storage_rebate >= cost {
            return BigUint::from(0u32);
        }
        computation_cost + storage_cost - storage_rebate
    }

    pub fn map_transaction(chain: Chain, transaction: SuiTransaction, block_number: i64) -> Option<Transaction> {
        let balance_changes = transaction.balance_changes.unwrap_or_default();
        let effects = transaction.effects.clone();
        let hash = transaction.digest.clone();
        let fee = Self::get_fee(effects.gas_used.clone());
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
            let state = if effects.status.status == "success" {
                TransactionState::Confirmed
            } else {
                TransactionState::Failed
            };

            let transaction = Transaction::new(
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
            let stake = serde_json::from_value::<super::model::EventStake>(event_json).ok()?;

            let transaction = Transaction::new(
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
            let stake: super::model::EventUnstake = serde_json::from_value::<super::model::EventUnstake>(event_json).ok()?;
            let value = stake.principal_amount; // add reward amount

            let transaction = Transaction::new(
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
}
