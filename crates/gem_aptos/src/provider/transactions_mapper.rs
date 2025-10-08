use crate::models::{Transaction, TransactionResponse};
use crate::{
    APTOS_NATIVE_COIN, DELEGATION_POOL_ADD_STAKE_EVENT, DELEGATION_POOL_UNLOCK_STAKE_EVENT, FUNGIBLE_ASSET_DEPOSIT_EVENT, FUNGIBLE_ASSET_WITHDRAW_EVENT,
    STAKE_DEPOSIT_EVENT,
};
use chain_primitives::{BalanceDiff, SwapMapper};
use chrono::DateTime;
use num_bigint::{BigInt, BigUint};
use primitives::{AssetId, Chain, SwapProvider, Transaction as PrimitivesTransaction, TransactionState, TransactionType};
use std::error::Error;

pub fn map_transaction_broadcast(response: &TransactionResponse) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(message) = &response.message {
        return Err(message.clone().into());
    }

    response.hash.clone().ok_or_else(|| "Transaction response missing hash".into())
}

pub fn map_transactions(transactions: Vec<Transaction>) -> Vec<PrimitivesTransaction> {
    let mut transactions = transactions.into_iter().flat_map(map_transaction).collect::<Vec<_>>();

    transactions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    transactions
}

struct TransactionMeta {
    hash: String,
    sender: String,
    state: TransactionState,
    fee: String,
    created_at: DateTime<chrono::Utc>,
}

fn extract_meta(transaction: &Transaction) -> Option<TransactionMeta> {
    let hash = transaction.hash.clone().unwrap_or_default();
    let sender = transaction.sender.clone().unwrap_or_default();
    let state = if transaction.success {
        TransactionState::Confirmed
    } else {
        TransactionState::Failed
    };
    let gas_used = BigUint::from(transaction.gas_used.unwrap_or_default());
    let gas_unit_price = BigUint::from(transaction.gas_unit_price.unwrap_or_default());
    let fee = (gas_used * gas_unit_price).to_string();
    let created_at = DateTime::from_timestamp_micros(transaction.timestamp as i64)?;

    Some(TransactionMeta {
        hash,
        sender,
        state,
        fee,
        created_at,
    })
}

fn map_swap_transaction(transaction: Transaction, events: Vec<crate::models::Event>, chain: Chain) -> Option<PrimitivesTransaction> {
    let meta = extract_meta(&transaction)?;

    let withdraw_event = events.iter().find(|e| e.event_type == FUNGIBLE_ASSET_WITHDRAW_EVENT)?;
    let deposit_event = events.iter().find(|e| e.event_type == FUNGIBLE_ASSET_DEPOSIT_EVENT)?;
    let withdraw_amount = withdraw_event.get_amount()?;
    let deposit_amount = deposit_event.get_amount()?;

    let type_args = transaction.payload.as_ref()?.type_arguments.clone();
    if type_args.len() != 2 {
        return None;
    }

    let map_asset = |coin_type: &str| {
        if coin_type == APTOS_NATIVE_COIN {
            chain.as_asset_id()
        } else {
            AssetId::from_token(chain, coin_type)
        }
    };

    let from_asset = map_asset(&type_args[0]);
    let to_asset = map_asset(&type_args[1]);

    let balance_diffs = vec![
        BalanceDiff {
            asset_id: from_asset,
            from_value: None,
            to_value: None,
            diff: -BigInt::parse_bytes(withdraw_amount.as_bytes(), 10)?,
        },
        BalanceDiff {
            asset_id: to_asset,
            from_value: None,
            to_value: None,
            diff: BigInt::parse_bytes(deposit_amount.as_bytes(), 10)?,
        },
    ];

    let provider = events.iter().find(|e| e.event_type.contains("Swap")).and_then(|e| {
        if e.event_type.contains("pancakeswap") || e.event_type.contains("0xc7efb4076dbe143cbcd98cfaaa929ecfc8f299203dfff63b95ccb6bfe19850fa") {
            Some(SwapProvider::PancakeswapAptosV2.id().to_owned())
        } else {
            None
        }
    });

    let swap = SwapMapper::map_swap(&balance_diffs, &BigUint::from(0u8), &chain.as_asset_id(), provider)?;
    let metadata = serde_json::to_value(&swap).ok();
    let to = meta.sender.clone();

    Some(build_transaction(
        meta,
        chain.as_asset_id(),
        to,
        swap.from_value,
        TransactionType::Swap,
        metadata,
    ))
}

fn build_transaction(
    meta: TransactionMeta,
    asset_id: AssetId,
    to: String,
    value: String,
    transaction_type: TransactionType,
    metadata: Option<serde_json::Value>,
) -> PrimitivesTransaction {
    PrimitivesTransaction::new(
        meta.hash,
        asset_id.clone(),
        meta.sender,
        to,
        None,
        transaction_type,
        meta.state,
        meta.fee,
        asset_id,
        value,
        None,
        metadata,
        meta.created_at,
    )
}

pub fn map_transaction(transaction: Transaction) -> Option<PrimitivesTransaction> {
    let chain = Chain::Aptos;
    let events = transaction.clone().events.unwrap_or_default();
    let meta = extract_meta(&transaction)?;
    let asset_id = chain.as_asset_id();

    if events.iter().any(|e| e.event_type.contains("Swap")) {
        return map_swap_transaction(transaction, events, chain);
    }

    for event in &events {
        match event.event_type.as_str() {
            DELEGATION_POOL_ADD_STAKE_EVENT => {
                let data: crate::models::DelegationPoolAddStakeData = serde_json::from_value(event.data.clone()?).ok()?;
                return Some(build_transaction(
                    meta,
                    asset_id,
                    data.pool_address,
                    data.amount_added,
                    TransactionType::StakeDelegate,
                    None,
                ));
            }
            DELEGATION_POOL_UNLOCK_STAKE_EVENT => {
                let data: crate::models::DelegationPoolUnlockStakeData = serde_json::from_value(event.data.clone()?).ok()?;
                return Some(build_transaction(
                    meta,
                    asset_id,
                    data.pool_address,
                    data.amount_unlocked,
                    TransactionType::StakeUndelegate,
                    None,
                ));
            }
            _ => continue,
        }
    }

    if transaction.transaction_type.as_deref() == Some("user_transaction") && events.len() <= 4 {
        let deposit_event = events
            .iter()
            .find(|x| x.event_type == STAKE_DEPOSIT_EVENT || x.event_type == FUNGIBLE_ASSET_DEPOSIT_EVENT)?;

        let to = if deposit_event.event_type == FUNGIBLE_ASSET_DEPOSIT_EVENT {
            transaction.payload.as_ref()?.arguments.first()?.as_str()?.to_string()
        } else {
            deposit_event.guid.account_address.clone()
        };

        let value = deposit_event.get_amount()?;

        return Some(build_transaction(meta, asset_id, to, value, TransactionType::Transfer, None));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TransactionResponse;

    #[test]
    fn test_map_transaction_broadcast() {
        let response = TransactionResponse {
            hash: Some("0xabc123".to_string()),
            message: None,
            error_code: None,
            vm_error_code: None,
        };

        let result = map_transaction_broadcast(&response).unwrap();
        assert_eq!(result, "0xabc123");
    }

    #[test]
    fn test_map_transaction_broadcast_error() {
        let response = TransactionResponse {
            hash: None,
            message: Some("Invalid transaction: Type: Validation Code: MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS".to_string()),
            error_code: Some("vm_error".to_string()),
            vm_error_code: Some(14),
        };

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid transaction: Type: Validation Code: MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS"
        );
    }

    #[test]
    fn test_map_transaction_broadcast_from_testdata() {
        let response: TransactionResponse = serde_json::from_str(include_str!("../../testdata/invalid_transaction_response.json")).unwrap();

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid transaction: Type: Validation Code: MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS"
        );
    }

    #[test]
    fn test_map_transaction_near_intent_transfer() {
        let transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_near_intent_transfer.json")).unwrap();

        let result = map_transaction(transaction);

        assert!(result.is_some());
        let tx = result.unwrap();
        assert_eq!(tx.hash, "0x6a43e0034486583a30cff449c03c4d882c641b351e392096272496168240de8e");
        assert_eq!(tx.from, "0xd1a1c1804e91ba85a569c7f018bb7502d2f13d4742d2611953c9c14681af6446");
        assert_eq!(tx.to, "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc");
        assert_eq!(tx.value, "2431838058");
        assert_eq!(tx.state, TransactionState::Confirmed);
        assert_eq!(tx.transaction_type, TransactionType::Transfer);
    }

    #[test]
    fn test_map_transaction_swap_pancakeswap() {
        let transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_swap_pancakeswap.json")).unwrap();

        let result = map_transaction(transaction);

        assert!(result.is_some());
        let tx = result.unwrap();
        assert_eq!(tx.hash, "0x0a91279f5e94dd678c2a1655856270fc7635c9c98bdb0b923ab2d50ad656ad7b");
        assert_eq!(tx.from, "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc");
        assert_eq!(tx.to, "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc");
        assert_eq!(tx.state, TransactionState::Confirmed);
        assert_eq!(tx.transaction_type, TransactionType::Swap);
        assert_eq!(tx.fee, "5700");
        assert!(tx.metadata.is_some());

        let metadata: primitives::TransactionSwapMetadata = serde_json::from_value(tx.metadata.unwrap()).unwrap();
        assert_eq!(metadata.from_asset, Chain::Aptos.as_asset_id());
        assert_eq!(metadata.from_value, "100000000");
        assert_eq!(
            metadata.to_asset.token_id.unwrap(),
            "0x159df6b7689437016108a019fd5bef736bac692b6d4a1f10c941f6fbb9a74ca6::oft::CakeOFT"
        );
        assert_eq!(metadata.to_value, "117926015");
        assert_eq!(metadata.provider.unwrap(), "pancakeswap_aptos_v2");
    }

    #[test]
    fn test_map_transaction_stake_delegate() {
        let transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_stake_delegate.json")).unwrap();

        let result = map_transaction(transaction);

        assert!(result.is_some());
        let tx = result.unwrap();
        assert_eq!(tx.hash, "0x130cc74c1a768780ca062a97bc833a01dec85b2d315484869559b7cdee4d0e75");
        assert_eq!(tx.from, "0xc95615aa095c100b18eb6eaa0f0a0f30b9cd96685118a7cbc1a2328a91ca2eda");
        assert_eq!(tx.to, "0xe5452230b8d5f4a664e33b8ad95354e50da64caaf003f11c0158391e96a4db2c");
        assert_eq!(tx.value, "1100000000");
        assert_eq!(tx.state, TransactionState::Confirmed);
        assert_eq!(tx.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(tx.fee, "142400");
    }

    #[test]
    fn test_map_transaction_stake_undelegate() {
        let transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_stake_undelegate.json")).unwrap();

        let result = map_transaction(transaction);

        assert!(result.is_some());
        let tx = result.unwrap();
        assert_eq!(tx.hash, "0xef6430bef0e8de7090b2c4bce210adb75d648be4614dcc37232b0d67f819b137");
        assert_eq!(tx.from, "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc");
        assert_eq!(tx.to, "0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e");
        assert_eq!(tx.value, "1109984251");
        assert_eq!(tx.state, TransactionState::Confirmed);
        assert_eq!(tx.transaction_type, TransactionType::StakeUndelegate);
        assert_eq!(tx.fee, "88400");
    }
}
