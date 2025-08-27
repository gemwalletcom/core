use crate::models::rpc::{
    AccountLedger, AccountLedgerTransaction, AccountObject, Amount, Ledger, Transaction as XrpTransaction, TransactionBroadcast, TransactionStatus,
};
use crate::{RESULT_SUCCESS, TRANSACTION_TYPE_PAYMENT, XRP_DEFAULT_ASSET_DECIMALS, XRP_EPOCH_OFFSET_SECONDS};
use chrono::DateTime;
use num_bigint::BigInt;
use primitives::{Asset, AssetId, AssetType, Transaction, TransactionChange, TransactionState, TransactionType, TransactionUpdate, chain::Chain};
use std::error::Error;

pub fn map_transaction_broadcast(broadcast_result: &TransactionBroadcast) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(accepted) = broadcast_result.accepted
        && !accepted
    {
        if let Some(error_msg) = &broadcast_result.engine_result_message {
            return Err(format!("Transaction rejected: {}", error_msg).into());
        }
        return Err("Transaction was not accepted".into());
    }

    if let Some(hash) = &broadcast_result.hash {
        Ok(hash.clone())
    } else if let Some(tx_json) = &broadcast_result.tx_json {
        Ok(tx_json.hash.clone())
    } else {
        Err("Transaction broadcast failed - no hash returned".into())
    }
}

pub fn map_transaction_status(status: &TransactionStatus) -> TransactionUpdate {
    let state = match status.status.as_str() {
        "success" => TransactionState::Confirmed,
        "failed" => TransactionState::Failed,
        _ => TransactionState::Pending,
    };

    let changes = vec![TransactionChange::NetworkFee(BigInt::from(status.fee.clone()))];

    TransactionUpdate { state, changes }
}

pub fn map_transactions_by_block(ledger: crate::models::rpc::Ledger) -> Vec<Transaction> {
    map_block_transactions(Chain::Xrp, ledger)
}

pub fn map_transactions_by_address(account_ledger: crate::models::rpc::AccountLedger) -> Vec<Transaction> {
    map_account_transactions(Chain::Xrp, account_ledger)
}

pub fn map_block_transactions(chain: Chain, ledger: Ledger) -> Vec<Transaction> {
    ledger
        .transactions
        .into_iter()
        .flat_map(|x| map_block_transaction(chain, x, ledger.close_time))
        .collect::<Vec<Transaction>>()
}

pub fn map_account_transactions(chain: Chain, ledger: AccountLedger) -> Vec<Transaction> {
    ledger
        .transactions
        .into_iter()
        .flat_map(|x| map_account_transaction(chain, x))
        .collect::<Vec<Transaction>>()
}

fn map_transaction_common(
    chain: Chain,
    hash: String,
    account: Option<String>,
    destination: Option<String>,
    amount: Option<Amount>,
    destination_tag: Option<i64>,
    fee: Option<String>,
    transaction_type: String,
    meta_result: String,
    timestamp: i64,
) -> Option<Transaction> {
    if transaction_type == TRANSACTION_TYPE_PAYMENT {
        let memo = destination_tag.map(|x| x.to_string());
        let value = amount.clone()?.as_value_string()?;
        let token_id = amount?.token_id();
        let asset_id = AssetId::from(chain, token_id);
        let created_at = DateTime::from_timestamp(timestamp, 0)?;

        let state = if meta_result == RESULT_SUCCESS {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };

        return Some(Transaction::new(
            hash,
            asset_id.clone(),
            account.unwrap_or_default(),
            destination.unwrap_or_default(),
            None,
            TransactionType::Transfer,
            state,
            fee.unwrap_or_default(),
            chain.as_asset_id(),
            value,
            memo,
            None,
            created_at,
        ));
    }
    None
}

pub fn map_account_transaction(chain: Chain, transaction: AccountLedgerTransaction) -> Option<Transaction> {
    map_transaction_common(
        chain,
        transaction.hash,
        transaction.tx_json.account,
        transaction.tx_json.destination,
        transaction.tx_json.amount,
        transaction.tx_json.destination_tag,
        transaction.tx_json.fee,
        transaction.tx_json.transaction_type,
        transaction.meta.result,
        XRP_EPOCH_OFFSET_SECONDS + transaction.tx_json.date,
    )
}

pub fn map_block_transaction(chain: Chain, transaction: XrpTransaction, close_time: i64) -> Option<Transaction> {
    map_transaction_common(
        chain,
        transaction.hash,
        transaction.account,
        transaction.destination,
        transaction.amount,
        transaction.destination_tag,
        transaction.fee,
        transaction.transaction_type,
        transaction.meta.result,
        XRP_EPOCH_OFFSET_SECONDS + close_time,
    )
}

pub fn map_token_data(chain: Chain, account_objects: Vec<AccountObject>) -> Result<Asset, Box<dyn Error + Send + Sync>> {
    let account = account_objects.first().ok_or("No account objects found for token_id")?;
    let symbol = account.low_limit.symbol().ok_or("Invalid currency")?;
    let token_id = &account.low_limit.issuer;

    Ok(Asset::new(
        AssetId::from_token(chain, token_id),
        symbol.clone(),
        symbol.clone(),
        XRP_DEFAULT_ASSET_DECIMALS as i32,
        AssetType::TOKEN,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rpc::{LedgerData, LedgerResult, TransactionBroadcast, TransactionStatus};
    use num_bigint::BigUint;

    #[test]
    fn test_map_transaction_broadcast_success() {
        let json_data = include_str!("../testdata/transaction_broadcast_success.json");
        let response: LedgerResult<TransactionBroadcast> = serde_json::from_str(json_data).expect("Failed to parse JSON");

        let result = map_transaction_broadcast(&response.result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "04F53F220DD1BCB7CCF279D66FFB986EA41383EFC9378CA1EBF1823D7C89264F");
    }

    #[test]
    fn test_map_transaction_broadcast_failed() {
        let json_data = include_str!("../testdata/transaction_broadcast_failed.json");
        let response: LedgerResult<TransactionBroadcast> = serde_json::from_str(json_data).expect("Failed to parse JSON");

        let result = map_transaction_broadcast(&response.result);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction rejected: Ledger sequence too high.");
    }

    #[test]
    fn test_map_transaction_status_success() {
        let status = TransactionStatus {
            status: "success".to_string(),
            fee: BigUint::from(100u64),
        };

        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes.len(), 1);
        if let TransactionChange::NetworkFee(fee) = &result.changes[0] {
            assert_eq!(fee, &BigInt::from(100u64));
        }
    }

    #[test]
    fn test_map_transaction_status_failed() {
        let status = TransactionStatus {
            status: "failed".to_string(),
            fee: BigUint::from(50u64),
        };

        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Failed);
        assert_eq!(result.changes.len(), 1);
        if let TransactionChange::NetworkFee(fee) = &result.changes[0] {
            assert_eq!(fee, &BigInt::from(50u64));
        }
    }

    #[test]
    fn test_map_transaction_status_pending() {
        let status = TransactionStatus {
            status: "pending".to_string(),
            fee: BigUint::from(75u64),
        };

        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Pending);
        assert_eq!(result.changes.len(), 1);
        if let TransactionChange::NetworkFee(fee) = &result.changes[0] {
            assert_eq!(fee, &BigInt::from(75u64));
        }
    }

    #[test]
    fn test_map_account_transactions() {
        let test_data = {
            let response: serde_json::Value = serde_json::from_str(include_str!("../testdata/account_transactions.json")).unwrap();
            serde_json::from_str::<AccountLedger>(&response["result"].to_string()).unwrap()
        };
        let transactions = map_account_transactions(Chain::Xrp, test_data);

        let expected_tx = Transaction::new(
            "00778C36255A48E753E7CDD3B60243D551ACD4B6ABD6765E9011D28B7566FEAB".to_string(),
            Chain::Xrp.as_asset_id(),
            "rGBpbVC11etyeGpJCAPrfS1of7SrEM2Q2c".to_string(),
            "rnZmVGX6f4pUYyS4oXYJzoLdRojQV8y297".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            "11".to_string(),
            Chain::Xrp.as_asset_id(),
            "1".to_string(),
            None,
            None,
            DateTime::from_timestamp(1749150631, 0).unwrap(),
        );

        assert_eq!(transactions.first().unwrap(), &expected_tx);
    }

    #[test]
    fn test_map_transactions_by_block() {
        let response: LedgerResult<LedgerData> = serde_json::from_str(include_str!("../testdata/transactions_by_block.json")).unwrap();
        let transactions = map_transactions_by_block(response.result.ledger);

        assert!(!transactions.is_empty());
        for tx in transactions {
            assert_eq!(tx.asset_id.chain, Chain::Xrp);
            assert_eq!(tx.transaction_type, TransactionType::Transfer);
        }
    }
}
