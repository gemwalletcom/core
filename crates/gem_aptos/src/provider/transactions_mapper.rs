use crate::models::{Transaction, TransactionResponse};
use crate::{APTOS_NATIVE_COIN, FUNGIBLE_ASSET_DEPOSIT_EVENT, FUNGIBLE_ASSET_WITHDRAW_EVENT, STAKE_DEPOSIT_EVENT};
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

fn map_swap_transaction(transaction: Transaction, events: Vec<crate::models::Event>, chain: Chain) -> Option<PrimitivesTransaction> {
    let hash = transaction.hash.clone().unwrap_or_default();
    let sender = transaction.sender.clone().unwrap_or_default();
    let state = if transaction.success {
        TransactionState::Confirmed
    } else {
        TransactionState::Failed
    };
    let gas_used = BigUint::from(transaction.gas_used.unwrap_or_default());
    let gas_unit_price = BigUint::from(transaction.gas_unit_price.unwrap_or_default());
    let fee = gas_used * gas_unit_price;
    let created_at = DateTime::from_timestamp_micros(transaction.timestamp as i64)?;

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

    Some(PrimitivesTransaction::new(
        hash,
        chain.as_asset_id(),
        sender.clone(),
        sender,
        None,
        TransactionType::Swap,
        state,
        fee.to_string(),
        chain.as_asset_id(),
        swap.from_value.clone(),
        None,
        serde_json::to_value(swap).ok(),
        created_at,
    ))
}

pub fn map_transaction(transaction: Transaction) -> Option<PrimitivesTransaction> {
    let chain = Chain::Aptos;
    let events = transaction.clone().events.unwrap_or_default();

    if events.iter().any(|e| e.event_type.contains("Swap")) {
        return map_swap_transaction(transaction, events, chain);
    }

    if transaction.transaction_type.as_deref() == Some("user_transaction") && events.len() <= 4 {
        let deposit_event = events
            .iter()
            .find(|x| x.event_type == STAKE_DEPOSIT_EVENT || x.event_type == FUNGIBLE_ASSET_DEPOSIT_EVENT)?;

        let asset_id = chain.as_asset_id();
        let state = if transaction.success {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };

        let to = if deposit_event.event_type == FUNGIBLE_ASSET_DEPOSIT_EVENT {
            transaction.payload.as_ref()?.arguments.first()?.as_str()?.to_string()
        } else {
            deposit_event.guid.account_address.clone()
        };

        let value = &deposit_event.get_amount()?;
        let gas_used = BigUint::from(transaction.gas_used.unwrap_or_default());
        let gas_unit_price = BigUint::from(transaction.gas_unit_price.unwrap_or_default());
        let fee = gas_used * gas_unit_price;
        let created_at = DateTime::from_timestamp_micros(transaction.timestamp as i64)?;

        let transaction = PrimitivesTransaction::new(
            transaction.hash.unwrap_or_default(),
            asset_id.clone(),
            transaction.sender.unwrap_or_default(),
            to,
            None,
            TransactionType::Transfer,
            state,
            fee.to_string(),
            asset_id,
            value.clone(),
            None,
            None,
            created_at,
        );
        return Some(transaction);
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
}
