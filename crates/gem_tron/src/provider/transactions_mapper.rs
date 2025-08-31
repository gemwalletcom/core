use chrono::DateTime;
use num_bigint::{BigInt, BigUint};
use num_traits::Num;
use primitives::{chain::Chain, AssetId, Transaction, TransactionChange, TransactionState, TransactionType, TransactionUpdate};
use std::error::Error;

use crate::address::TronAddress;
use crate::models::{BlockTransactions, Transaction as TronTransaction, TransactionReceiptData, TronTransactionBroadcast};
use crate::rpc::constants::{ERC20_TRANSFER_EVENT_SIGNATURE, RECEIPT_FAILED, RECEIPT_OUT_OF_ENERGY};

const TRANSFER_CONTRACT: &str = "TransferContract";
const TRIGGER_SMART_CONTRACT: &str = "TriggerSmartContract";

fn decode_hex_message(hex_str: &str) -> String {
    match hex::decode(hex_str) {
        Ok(bytes) => String::from_utf8(bytes).unwrap_or_else(|_| hex_str.to_string()),
        Err(_) => hex_str.to_string(),
    }
}

pub fn map_transaction_broadcast(response: &TronTransactionBroadcast) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(message) = &response.message {
        Err(decode_hex_message(message).into())
    } else if let Some(txid) = &response.txid {
        Ok(txid.clone())
    } else {
        Err("Transaction broadcast failed with unknown error".into())
    }
}

pub fn map_transaction_status(receipt: &TransactionReceiptData) -> TransactionUpdate {
    if let Some(receipt_result) = &receipt.receipt.result {
        if receipt_result == RECEIPT_OUT_OF_ENERGY || receipt_result == RECEIPT_FAILED {
            return TransactionUpdate::new_state(TransactionState::Reverted);
        }
    }

    if receipt.block_number > 0 {
        let mut changes = vec![];
        if let Some(fee) = receipt.fee {
            changes.push(TransactionChange::NetworkFee(BigInt::from(fee)));
        }
        return TransactionUpdate::new(TransactionState::Confirmed, changes);
    }

    TransactionUpdate::new_state(TransactionState::Pending)
}

pub fn map_transactions_by_block(chain: Chain, block: BlockTransactions, receipts: Vec<TransactionReceiptData>) -> Vec<Transaction> {
    block
        .transactions
        .into_iter()
        .zip(receipts.iter())
        .filter_map(|(transaction, receipt)| map_transaction(chain, transaction, receipt.clone()))
        .collect()
}

pub fn map_transactions_by_address(transactions: Vec<TronTransaction>, receipts: Vec<TransactionReceiptData>) -> Vec<Transaction> {
    transactions
        .into_iter()
        .zip(receipts.iter())
        .filter_map(|(transaction, receipt)| map_transaction(Chain::Tron, transaction, receipt.clone()))
        .collect()
}

pub fn map_transaction(chain: Chain, transaction: TronTransaction, receipt: TransactionReceiptData) -> Option<Transaction> {
    if let (Some(value), Some(contract_result)) = (transaction.raw_data.contract.first().cloned(), transaction.ret.first().cloned()) {
        let state: TransactionState = if contract_result.contract_ret.clone() == "SUCCESS" {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };
        let fee = receipt.fee.unwrap_or_default().to_string();
        let created_at = DateTime::from_timestamp_millis(receipt.block_time_stamp)?;

        if value.contract_type == TRANSFER_CONTRACT && !transaction.ret.is_empty() {
            let from = TronAddress::from_hex(value.parameter.value.owner_address.unwrap_or_default().as_str()).unwrap_or_default();
            let to = TronAddress::from_hex(value.parameter.value.to_address.unwrap_or_default().as_str()).unwrap_or_default();

            let transaction = Transaction::new(
                transaction.tx_id,
                chain.as_asset_id(),
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                fee,
                chain.as_asset_id(),
                value.parameter.value.amount.unwrap_or_default().to_string(),
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }
        let logs = receipt.log.unwrap_or_default();
        if value.contract_type == TRIGGER_SMART_CONTRACT
            && logs.len() == 1
            && logs.first()?.topics.clone().unwrap_or_default().len() == 3
            && logs.first()?.topics.clone().unwrap_or_default().first()? == ERC20_TRANSFER_EVENT_SIGNATURE
        {
            let log = logs.first()?;
            let from_string = format!("41{}", log.topics.clone().unwrap_or_default()[1].clone().chars().skip(24).collect::<String>());
            let to_string = format!("41{}", log.topics.clone().unwrap_or_default()[2].clone().chars().skip(24).collect::<String>());
            let token_id = TronAddress::from_hex(value.parameter.value.contract_address?.as_str()).unwrap_or_default();
            let from = TronAddress::from_hex(from_string.as_str()).unwrap_or_default();
            let to = TronAddress::from_hex(to_string.as_str()).unwrap_or_default();
            let value = BigUint::from_str_radix(&log.data.clone().unwrap_or_default(), 16).unwrap();
            let asset_id = AssetId {
                chain,
                token_id: Some(token_id),
            };

            let transaction = Transaction::new(
                transaction.tx_id,
                asset_id,
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                fee,
                chain.as_asset_id(),
                value.to_string(),
                None,
                None,
                created_at,
            );

            return Some(transaction);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TransactionReceipt, TransactionReceiptData, TronTransactionBroadcast};

    #[test]
    fn test_map_transaction_broadcast_error() {
        let response: TronTransactionBroadcast = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_error.json")).unwrap();

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Contract validate error : Cannot transfer TRX to yourself.");
    }

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response: TronTransactionBroadcast = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_success.json")).unwrap();

        let result = map_transaction_broadcast(&response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "7f60ccd0594b5c3e0264cca9a6e6e64cb96ee66ce3a796b4356cb8ccc548f62b");
    }

    #[test]
    fn test_map_transaction_broadcast_unknown_error() {
        let response = TronTransactionBroadcast {
            txid: None,
            code: None,
            message: None,
        };

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction broadcast failed with unknown error");
    }

    #[test]
    fn test_map_transaction_status_confirmed() {
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1234567890,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
        };

        let result = map_transaction_status(&receipt);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes.len(), 1);
        if let TransactionChange::NetworkFee(fee) = &result.changes[0] {
            assert_eq!(fee, &BigInt::from(1000i64));
        }
    }

    #[test]
    fn test_map_transaction_status_reverted_out_of_energy() {
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(500),
            block_number: 12345,
            block_time_stamp: 1234567890,
            receipt: TransactionReceipt {
                result: Some(RECEIPT_OUT_OF_ENERGY.to_string()),
            },
            log: None,
        };

        let result = map_transaction_status(&receipt);
        assert_eq!(result.state, TransactionState::Reverted);
        assert_eq!(result.changes.len(), 0);
    }

    #[test]
    fn test_map_transaction_status_reverted_failed() {
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(250),
            block_number: 12345,
            block_time_stamp: 1234567890,
            receipt: TransactionReceipt {
                result: Some(RECEIPT_FAILED.to_string()),
            },
            log: None,
        };

        let result = map_transaction_status(&receipt);
        assert_eq!(result.state, TransactionState::Reverted);
        assert_eq!(result.changes.len(), 0);
    }

    #[test]
    fn test_map_transaction_status_pending() {
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(0),
            block_number: 0,
            block_time_stamp: 0,
            receipt: TransactionReceipt { result: None },
            log: None,
        };

        let result = map_transaction_status(&receipt);
        assert_eq!(result.state, TransactionState::Pending);
        assert_eq!(result.changes.len(), 0);
    }
}
