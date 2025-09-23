use crate::models::{BalanceChange, Digest, EventStake, EventUnstake, GasUsed, TransactionBlocks};
use crate::{SUI_COIN_TYPE, SUI_COIN_TYPE_FULL, SUI_STAKE_EVENT, SUI_UNSTAKE_EVENT};
use chain_primitives::{BalanceDiff, SwapMapper};
use chrono::{TimeZone, Utc};
use num_bigint::BigUint;
use primitives::{AssetId, SwapProvider, Transaction, TransactionState, TransactionSwapMetadata, TransactionType, TransactionUpdate, chain::Chain};

const CHAIN: Chain = Chain::Sui;

pub fn get_fee(gas_used: GasUsed) -> BigUint {
    let computation_cost = gas_used.computation_cost;
    let storage_cost = gas_used.storage_cost;
    let storage_rebate = gas_used.storage_rebate;

    let cost = computation_cost.clone() + storage_cost.clone();
    if storage_rebate >= cost {
        return BigUint::from(0u32);
    }
    computation_cost + storage_cost - storage_rebate
}

pub fn map_transaction(transaction: Digest) -> Option<Transaction> {
    let chain = CHAIN;
    let balance_changes = transaction.balance_changes.unwrap_or_default();
    let effects = transaction.effects.clone();
    let hash = transaction.digest.clone();
    let fee = get_fee(effects.gas_used.clone());
    let created_at = Utc.timestamp_millis_opt(transaction.timestamp_ms as i64).unwrap();
    let state = if effects.status.status == "success" {
        TransactionState::Confirmed
    } else {
        TransactionState::Failed
    };
    let owner = effects.gas_object.owner.get_address_owner();
    let sui_coin_type = chain.as_denom()?;

    // system & token transfer
    if transaction.events.is_empty() && (balance_changes.len() == 2 || balance_changes.len() == 3) {
        let (from_change, to_change) =
            if balance_changes.len() == 2 && balance_changes[0].coin_type == sui_coin_type && balance_changes[1].coin_type == sui_coin_type {
                if balance_changes[0].amount < balance_changes[1].amount {
                    (balance_changes[0].clone(), balance_changes[1].clone())
                } else {
                    (balance_changes[1].clone(), balance_changes[0].clone())
                }
            } else if balance_changes.len() == 3 && balance_changes[0].coin_type == sui_coin_type {
                if balance_changes[1].amount < balance_changes[2].amount {
                    (balance_changes[1].clone(), balance_changes[2].clone())
                } else {
                    (balance_changes[2].clone(), balance_changes[1].clone())
                }
            } else {
                return None;
            };

        let asset_id = if from_change.coin_type == sui_coin_type {
            chain.as_asset_id()
        } else {
            AssetId::from_token(chain, &from_change.coin_type)
        };
        let from = from_change.owner.get_address_owner()?;
        let to = to_change.owner.get_address_owner()?;
        let value = to_change.amount;

        let transaction = Transaction::new(
            hash,
            asset_id,
            from,
            to,
            None,
            TransactionType::Transfer,
            state,
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
    if transaction.events.len() == 1 && transaction.events.first().is_some_and(|e| e.event_type == SUI_STAKE_EVENT) {
        let event = transaction.events.first()?;
        let event_json = event.parsed_json.clone()?;
        let stake = serde_json::from_value::<EventStake>(event_json).ok()?;

        let transaction = Transaction::new(
            hash,
            chain.as_asset_id(),
            stake.staker_address,
            stake.validator_address,
            None,
            TransactionType::StakeDelegate,
            state,
            fee.to_string(),
            chain.as_asset_id(),
            stake.amount,
            None,
            None,
            created_at,
        );
        return Some(transaction);
    }

    // swap
    if transaction.events.iter().any(|x| x.event_type.contains("Swap")) {
        let owner_balance_changes = balance_changes
            .iter()
            .filter(|x| x.owner.get_address_owner() == owner)
            .cloned()
            .collect::<Vec<_>>();
        // TODO: Handle other swap providers
        let swap = match owner_balance_changes.len() {
            2 => map_swap_from_balance_changes(owner_balance_changes.clone(), &fee)?,
            3 => {
                let owner_balance_changes_filtered = owner_balance_changes
                    .iter()
                    .filter(|x| x.coin_type != SUI_COIN_TYPE)
                    .cloned()
                    .collect::<Vec<_>>();
                map_swap_from_balance_changes(owner_balance_changes_filtered.clone(), &fee)?
            }
            _ => return None,
        };

        let transaction = Transaction::new(
            hash,
            chain.as_asset_id(),
            owner.clone()?,
            owner.clone()?,
            None,
            TransactionType::Swap,
            TransactionState::Confirmed,
            fee.to_string(),
            chain.as_asset_id(),
            swap.clone().from_value,
            None,
            serde_json::to_value(swap.clone()).ok(),
            created_at,
        );
        return Some(transaction);
    }

    // unstake
    if transaction.events.len() == 1 && transaction.events.first().is_some_and(|e| e.event_type == SUI_UNSTAKE_EVENT) {
        let event = transaction.events.first()?;
        let event_json = event.parsed_json.clone()?;
        let stake: EventUnstake = serde_json::from_value::<EventUnstake>(event_json).ok()?;
        let value = stake.principal_amount; // add reward amount

        let transaction = Transaction::new(
            hash,
            chain.as_asset_id(),
            stake.staker_address,
            stake.validator_address,
            None,
            TransactionType::StakeUndelegate,
            state,
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

pub fn map_swap_from_balance_changes(balance_changes: Vec<BalanceChange>, fee: &BigUint) -> Option<TransactionSwapMetadata> {
    let balance_diffs: Vec<BalanceDiff> = balance_changes
        .into_iter()
        .map(|change| BalanceDiff {
            asset_id: map_asset_id(&change.coin_type),
            from_value: None,
            to_value: None,
            diff: change.amount,
        })
        .collect();

    let native_asset_id = Chain::Sui.as_asset_id();
    SwapMapper::map_swap(&balance_diffs, fee, &native_asset_id, Some(SwapProvider::Cetus.id().to_owned()))
}

pub fn map_asset_id(coin_type: &str) -> AssetId {
    match coin_type {
        SUI_COIN_TYPE | SUI_COIN_TYPE_FULL => Chain::Sui.as_asset_id(),
        _ => AssetId::from_token(Chain::Sui, coin_type),
    }
}

pub fn map_transaction_blocks(transaction_blocks: TransactionBlocks) -> Vec<Transaction> {
    transaction_blocks.data.into_iter().flat_map(map_transaction).collect()
}

pub fn map_transaction_status(transaction: Digest) -> TransactionUpdate {
    let state = match transaction.effects.status.status.as_str() {
        "success" => TransactionState::Confirmed,
        "failure" => TransactionState::Reverted,
        _ => TransactionState::Pending,
    };
    TransactionUpdate::new_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_transaction_blocks() {
        let transaction_blocks = TransactionBlocks { data: vec![] };
        let transactions = map_transaction_blocks(transaction_blocks);
        assert_eq!(transactions.len(), 0);
    }

    #[test]
    fn test_map_transaction_status() {
        use crate::models::{Effect, GasObject, GasUsed, Owner, Status};
        use num_bigint::BigUint;

        let digest = Digest {
            digest: "test".to_string(),
            effects: Effect {
                gas_used: GasUsed {
                    computation_cost: BigUint::from(1000u32),
                    storage_cost: BigUint::from(500u32),
                    storage_rebate: BigUint::from(100u32),
                    non_refundable_storage_fee: BigUint::from(0u32),
                },
                status: Status { status: "success".to_string() },
                gas_object: GasObject {
                    owner: Owner::String("0x123".to_string()),
                },
            },
            balance_changes: None,
            events: vec![],
            timestamp_ms: 1234567890,
        };

        let update = map_transaction_status(digest);
        assert_eq!(update.state, TransactionState::Confirmed);
    }
}
