use crate::models::{BalanceChange, Digest, Event, EventStake, EventUnstake, GasUsed, TransactionBlocks};
use crate::{SUI_COIN_TYPE, SUI_STAKE_EVENT, SUI_UNSTAKE_EVENT, full_coin_type};
use chain_primitives::{BalanceDiff, SwapMapper};
use chrono::{TimeZone, Utc};
use num_bigint::{BigUint, Sign};
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

    let (asset_id, from, to, transaction_type, value, metadata) = map_transaction_type(&transaction.events, &balance_changes, &owner, &fee)?;

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
    events: &[Event],
    balance_changes: &[BalanceChange],
    owner: &Option<String>,
    fee: &BigUint,
) -> Option<(AssetId, String, String, TransactionType, String, Option<serde_json::Value>)> {
    let chain = CHAIN;

    // system & token transfer
    if events.is_empty() && (balance_changes.len() == 2 || balance_changes.len() == 3) {
        let (from_change, to_change) = map_transfer_balance_changes(balance_changes, fee)?;

        let asset_id = if is_native_sui(&from_change.coin_type) {
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
    if let Some(event) = single_event(events, SUI_STAKE_EVENT) {
        let event_json = event.parsed_json.clone()?;
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
                let filtered: Vec<_> = owner_balance_changes.into_iter().filter(|x| !is_native_sui(&x.coin_type)).collect();
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
    if let Some(event) = single_event(events, SUI_UNSTAKE_EVENT) {
        let event_json = event.parsed_json.clone()?;
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

fn map_transfer_balance_changes<'a>(balance_changes: &'a [BalanceChange], fee: &BigUint) -> Option<(&'a BalanceChange, &'a BalanceChange)> {
    let to_change = single(balance_changes.iter().filter(|change| change.amount.sign() == Sign::Plus))?;
    let from_change = single(outgoing_changes(balance_changes, &to_change.coin_type)).or_else(|| select_native_transfer_source(balance_changes, to_change, fee))?;
    Some((from_change, to_change))
}

fn single<T>(mut values: impl Iterator<Item = T>) -> Option<T> {
    let value = values.next()?;
    values.next().is_none().then_some(value)
}

fn outgoing_changes<'a>(balance_changes: &'a [BalanceChange], coin_type: &'a str) -> impl Iterator<Item = &'a BalanceChange> + 'a {
    balance_changes
        .iter()
        .filter(move |change| change.amount.sign() == Sign::Minus && type_tag_matches(&change.coin_type, coin_type))
}

fn select_native_transfer_source<'a>(balance_changes: &'a [BalanceChange], to_change: &'a BalanceChange, fee: &BigUint) -> Option<&'a BalanceChange> {
    if !is_native_sui(&to_change.coin_type) {
        return None;
    }

    let amount = to_change.amount.magnitude().clone();
    outgoing_changes(balance_changes, &to_change.coin_type)
        .find(|change| change.amount.magnitude() == &amount)
        .or_else(|| {
            let amount_with_fee = amount + fee;
            outgoing_changes(balance_changes, &to_change.coin_type).find(|change| change.amount.magnitude() == &amount_with_fee)
        })
        .or_else(|| outgoing_changes(balance_changes, &to_change.coin_type).max_by(|left, right| left.amount.magnitude().cmp(right.amount.magnitude())))
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
    SwapMapper::map_swap(&balance_diffs, fee, &native_asset_id, Some(SwapProvider::CetusClmm.id().to_owned()))
}

pub fn map_asset_id(coin_type: &str) -> AssetId {
    if is_native_sui(coin_type) {
        Chain::Sui.as_asset_id()
    } else {
        AssetId::from_token(Chain::Sui, coin_type)
    }
}

fn is_native_sui(coin_type: &str) -> bool {
    type_tag_matches(coin_type, SUI_COIN_TYPE)
}

fn single_event<'a>(events: &'a [Event], event_type: &str) -> Option<&'a Event> {
    let [event] = events else {
        return None;
    };
    type_tag_matches(&event.event_type, event_type).then_some(event)
}

fn type_tag_matches(value: &str, expected: &str) -> bool {
    full_coin_type(value) == full_coin_type(expected)
}

pub fn map_transaction_blocks(transaction_blocks: TransactionBlocks) -> Vec<Transaction> {
    transaction_blocks.data.into_iter().flat_map(map_transaction).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Effect, GasObject, Owner, OwnerObject, Status};
    use crate::provider::testkit::TEST_TRANSACTION_ID;
    use crate::{SUI_COIN_TYPE_FULL, SUI_UNSTAKE_EVENT};
    use num_bigint::{BigInt, BigUint};
    use serde_json::json;

    const OWNER_ADDRESS: &str = "0x1930a5e729ad95a48e4d9dc2ca8a001f8ed18b20077c083cd6b1d3355a7972a5";
    const RECIPIENT_ADDRESS: &str = "0x9d6b98b18fd26b5efeec68d020dcf1be7a94c2c315353779bc6b3aed44188ddf";
    const SPONSORED_TRANSFER_SENDER_ADDRESS: &str = "0x00ea18889868519abd2f238966cab9875750bb2859ed3a34debec37781520138";
    const VALIDATOR_ADDRESS: &str = "0xbba318294a51ddeafa50c335c8e77202170e1f272599a2edc40592100863f638";
    const TOKEN_A: &str = "0x00000000000000000000000000000000000000000000000000000000000000aa::coin::AAA";
    const TOKEN_B: &str = "0x00000000000000000000000000000000000000000000000000000000000000bb::coin::BBB";

    fn owner(address: &str) -> Owner {
        Owner::OwnerObject(OwnerObject {
            address_owner: Some(address.to_string()),
        })
    }

    fn balance_change(address: &str, coin_type: &str, amount: i64) -> BalanceChange {
        BalanceChange {
            owner: owner(address),
            coin_type: coin_type.to_string(),
            amount: BigInt::from(amount),
        }
    }

    fn event(event_type: impl Into<String>, parsed_json: serde_json::Value) -> Event {
        Event {
            event_type: event_type.into(),
            parsed_json: Some(parsed_json),
            package_id: String::new(),
        }
    }

    fn make_digest(events: Vec<Event>, balance_changes: Vec<BalanceChange>) -> Digest {
        Digest {
            digest: "test".to_string(),
            effects: Effect {
                gas_used: GasUsed {
                    computation_cost: BigUint::from(0u32),
                    storage_cost: BigUint::from(0u32),
                    storage_rebate: BigUint::from(0u32),
                    non_refundable_storage_fee: BigUint::from(0u32),
                },
                status: Status { status: "success".to_string() },
                gas_object: GasObject { owner: owner(OWNER_ADDRESS) },
            },
            balance_changes: Some(balance_changes),
            events,
            timestamp_ms: 1778964551487,
        }
    }

    #[test]
    fn test_map_transaction_blocks() {
        let transaction_blocks = TransactionBlocks { data: vec![] };
        let transactions = map_transaction_blocks(transaction_blocks);
        assert_eq!(transactions.len(), 0);
    }

    #[test]
    fn test_map_smart_contract_call() {
        let digest: Digest = serde_json::from_str(include_str!("../../testdata/transfer_token_contract.json")).unwrap();
        let transaction = map_transaction(digest).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(transaction.value, "0");

        let metadata: TransactionSmartContractMetadata = serde_json::from_value(transaction.metadata.unwrap()).unwrap();
        assert_eq!(metadata.method_name, "timevy_tipping");
    }

    #[test]
    fn test_map_transaction_by_hash() {
        let digest: Digest = serde_json::from_str(include_str!("../../testdata/transfer_sui.json")).unwrap();
        let transaction = map_transaction(digest).unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
    }

    #[test]
    fn test_map_full_type_tags() {
        let digest: Digest = serde_json::from_str(include_str!("../../testdata/stake_grpc.json")).unwrap();
        let transaction = map_transaction(digest).unwrap();

        assert_eq!(transaction.hash, "DXKezMGJZaxJRC6a6zCr3JdfquYGxgU1zjV4xrNAaCFB");
        assert_eq!(transaction.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(transaction.value, "2000000000");
        assert_eq!(transaction.from, OWNER_ADDRESS);
        assert_eq!(transaction.to, VALIDATOR_ADDRESS);
        assert_eq!(transaction.fee, "10610996");
        assert_eq!(map_asset_id(SUI_COIN_TYPE_FULL), Chain::Sui.as_asset_id());

        let native_transfer = map_transaction(make_digest(
            vec![],
            vec![
                balance_change(OWNER_ADDRESS, SUI_COIN_TYPE_FULL, -101744880),
                balance_change(RECIPIENT_ADDRESS, SUI_COIN_TYPE_FULL, 100000000),
            ],
        ))
        .unwrap();

        assert_eq!(native_transfer.transaction_type, TransactionType::Transfer);
        assert_eq!(native_transfer.asset_id, Chain::Sui.as_asset_id());
        assert_eq!(native_transfer.value, "100000000");

        let digest: Digest = serde_json::from_str(include_str!("../../testdata/sponsored_transfer_sui.json")).unwrap();
        let sponsored_transfer = map_transaction(digest).unwrap();

        assert_eq!(sponsored_transfer.transaction_type, TransactionType::Transfer);
        assert_eq!(sponsored_transfer.asset_id, Chain::Sui.as_asset_id());
        assert_eq!(sponsored_transfer.from, SPONSORED_TRANSFER_SENDER_ADDRESS);
        assert_eq!(sponsored_transfer.to, OWNER_ADDRESS);
        assert_eq!(sponsored_transfer.value, "5996594751");

        let token_transfer = map_transaction(make_digest(
            vec![],
            vec![
                balance_change(OWNER_ADDRESS, SUI_COIN_TYPE_FULL, -1000),
                balance_change(OWNER_ADDRESS, TOKEN_A, -100),
                balance_change(RECIPIENT_ADDRESS, TOKEN_A, 100),
            ],
        ))
        .unwrap();

        assert_eq!(token_transfer.transaction_type, TransactionType::Transfer);
        assert_eq!(token_transfer.asset_id, AssetId::from_token(Chain::Sui, TOKEN_A));
        assert_eq!(token_transfer.value, "100");

        let swap = map_transaction(make_digest(
            vec![event("0x00000000000000000000000000000000000000000000000000000000000000cc::pool::SwapEvent", json!({}))],
            vec![
                balance_change(OWNER_ADDRESS, SUI_COIN_TYPE_FULL, -1000),
                balance_change(OWNER_ADDRESS, TOKEN_A, -200),
                balance_change(OWNER_ADDRESS, TOKEN_B, 150),
            ],
        ))
        .unwrap();

        assert_eq!(swap.transaction_type, TransactionType::Swap);
        assert_eq!(swap.value, "200");
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap.metadata.unwrap()).unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_token(Chain::Sui, TOKEN_A));
        assert_eq!(metadata.from_value, "200");
        assert_eq!(metadata.to_asset, AssetId::from_token(Chain::Sui, TOKEN_B));
        assert_eq!(metadata.to_value, "150");

        let unstake = map_transaction(make_digest(
            vec![event(
                full_coin_type(SUI_UNSTAKE_EVENT),
                json!({
                    "principal_amount": "3000000000",
                    "reward_amount": "42",
                    "staker_address": OWNER_ADDRESS,
                    "validator_address": VALIDATOR_ADDRESS,
                }),
            )],
            vec![balance_change(OWNER_ADDRESS, SUI_COIN_TYPE_FULL, 3000000000)],
        ))
        .unwrap();

        assert_eq!(unstake.transaction_type, TransactionType::StakeUndelegate);
        assert_eq!(unstake.value, "3000000000");
        assert_eq!(unstake.from, OWNER_ADDRESS);
        assert_eq!(unstake.to, VALIDATOR_ADDRESS);
    }
}
