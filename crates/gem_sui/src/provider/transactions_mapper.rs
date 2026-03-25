use crate::models::{BalanceChange, Digest, EventStake, EventUnstake, GasUsed, TransactionBlocks};
use crate::{SUI_COIN_TYPE, SUI_COIN_TYPE_FULL, SUI_STAKE_EVENT, SUI_UNSTAKE_EVENT};
use chain_primitives::{BalanceDiff, SwapMapper};
use chrono::{TimeZone, Utc};
use num_bigint::BigUint;
use primitives::{AssetId, SwapProvider, Transaction, TransactionSmartContractMetadata, TransactionState, TransactionSwapMetadata, TransactionType, chain::Chain};

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

    let (asset_id, from, to, transaction_type, value, metadata) = map_transaction_type(&transaction.events, &balance_changes, &owner, sui_coin_type, &fee)?;

    Some(Transaction::new(
        hash,
        asset_id,
        from,
        to,
        None,
        transaction_type,
        state,
        fee.to_string(),
        chain.as_asset_id(),
        value,
        None,
        metadata,
        created_at,
    ))
}

fn map_transaction_type(
    events: &[crate::models::Event],
    balance_changes: &[BalanceChange],
    owner: &Option<String>,
    sui_coin_type: &str,
    fee: &BigUint,
) -> Option<(AssetId, String, String, TransactionType, String, Option<serde_json::Value>)> {
    let chain = CHAIN;

    // system & token transfer
    if events.is_empty() && (balance_changes.len() == 2 || balance_changes.len() == 3) {
        let (from_change, to_change) = if balance_changes.len() == 2 && balance_changes[0].coin_type == sui_coin_type && balance_changes[1].coin_type == sui_coin_type {
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
        return Some((
            asset_id,
            from_change.owner.get_address_owner()?,
            to_change.owner.get_address_owner()?,
            TransactionType::Transfer,
            to_change.amount.to_string(),
            None,
        ));
    }

    // stake
    if events.len() == 1 && events.first().is_some_and(|e| e.event_type == SUI_STAKE_EVENT) {
        let event_json = events.first()?.parsed_json.clone()?;
        let stake = serde_json::from_value::<EventStake>(event_json).ok()?;
        return Some((
            chain.as_asset_id(),
            stake.staker_address,
            stake.validator_address,
            TransactionType::StakeDelegate,
            stake.amount,
            None,
        ));
    }

    // swap
    if events.iter().any(|x| x.event_type.contains("Swap")) {
        let owner_balance_changes: Vec<_> = balance_changes.iter().filter(|x| x.owner.get_address_owner() == *owner).cloned().collect();
        let swap = match owner_balance_changes.len() {
            2 => map_swap_from_balance_changes(owner_balance_changes, fee)?,
            3 => {
                let filtered: Vec<_> = owner_balance_changes.into_iter().filter(|x| x.coin_type != SUI_COIN_TYPE).collect();
                map_swap_from_balance_changes(filtered, fee)?
            }
            _ => return None,
        };
        let owner = owner.clone()?;
        return Some((
            chain.as_asset_id(),
            owner.clone(),
            owner,
            TransactionType::Swap,
            swap.from_value.clone(),
            serde_json::to_value(&swap).ok(),
        ));
    }

    // unstake
    if events.len() == 1 && events.first().is_some_and(|e| e.event_type == SUI_UNSTAKE_EVENT) {
        let event_json = events.first()?.parsed_json.clone()?;
        let stake = serde_json::from_value::<EventUnstake>(event_json).ok()?;
        return Some((
            chain.as_asset_id(),
            stake.staker_address,
            stake.validator_address,
            TransactionType::StakeUndelegate,
            stake.principal_amount,
            None,
        ));
    }

    // smart contract call
    if !events.is_empty() {
        let method_name = events.first()?.event_type.rsplit("::").nth(1)?.to_string();
        let metadata = TransactionSmartContractMetadata { method_name };
        let owner = owner.clone()?;
        return Some((
            chain.as_asset_id(),
            owner.clone(),
            owner,
            TransactionType::SmartContractCall,
            "0".to_string(),
            serde_json::to_value(metadata).ok(),
        ));
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
    SwapMapper::map_swap(&balance_diffs, fee, &native_asset_id, Some(SwapProvider::CetusAggregator.id().to_owned()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::testkit::TEST_TRANSACTION_ID;

    #[test]
    fn test_map_transaction_blocks() {
        let transaction_blocks = TransactionBlocks { data: vec![] };
        let transactions = map_transaction_blocks(transaction_blocks);
        assert_eq!(transactions.len(), 0);
    }

    #[test]
    fn test_map_smart_contract_call() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/transfer_token_contract.json")).unwrap();
        let digest: Digest = serde_json::from_value(response["result"].clone()).unwrap();
        let transaction = map_transaction(digest).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(transaction.value, "0");

        let metadata: TransactionSmartContractMetadata = serde_json::from_value(transaction.metadata.unwrap()).unwrap();
        assert_eq!(metadata.method_name, "timevy_tipping");
    }

    #[test]
    fn test_map_transaction_by_hash() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/transfer_sui.json")).unwrap();
        let digest: Digest = serde_json::from_value(response["result"].clone()).unwrap();
        let transaction = map_transaction(digest).unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
    }
}
