use chrono::DateTime;
use num_bigint::BigUint;
use num_traits::Num;
use primitives::{AssetId, Transaction, TransactionState, TransactionType, chain::Chain};
use std::error::Error;

use crate::address::TronAddress;
use crate::models::{BlockTransactions, Transaction as TronTransaction, TransactionReceiptData, TronTransactionBroadcast};
use crate::rpc::constants::ERC20_TRANSFER_EVENT_SIGNATURE;

const TRANSFER_CONTRACT: &str = "TransferContract";
const TRIGGER_SMART_CONTRACT: &str = "TriggerSmartContract";
const FREEZE_BALANCE_V2_CONTRACT: &str = "FreezeBalanceV2Contract";
const UNFREEZE_BALANCE_V2_CONTRACT: &str = "UnfreezeBalanceV2Contract";
const VOTE_WITNESS_CONTRACT: &str = "VoteWitnessContract";

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

        let from = TronAddress::from_hex(value.parameter.value.owner_address.unwrap_or_default().as_str()).unwrap_or_default();

        if let Some((transaction_type, to, amount)) = match value.contract_type.as_str() {
            TRANSFER_CONTRACT if !transaction.ret.is_empty() => {
                let to = TronAddress::from_hex(value.parameter.value.to_address.unwrap_or_default().as_str()).unwrap_or_default();
                Some((TransactionType::Transfer, to, value.parameter.value.amount.unwrap_or_default().to_string()))
            }
            FREEZE_BALANCE_V2_CONTRACT => Some((
                TransactionType::StakeDelegate,
                from.clone(),
                value.parameter.value.frozen_balance.unwrap_or_default().to_string(),
            )),
            UNFREEZE_BALANCE_V2_CONTRACT => Some((
                TransactionType::StakeUndelegate,
                from.clone(),
                value.parameter.value.unfreeze_balance.unwrap_or_default().to_string(),
            )),
            VOTE_WITNESS_CONTRACT => Some((TransactionType::StakeDelegate, from.clone(), "0".to_string())),
            _ => None,
        } {
            let transaction = Transaction::new(
                transaction.tx_id,
                chain.as_asset_id(),
                from,
                to,
                None,
                transaction_type,
                state,
                fee,
                chain.as_asset_id(),
                amount,
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
    fn test_map_transaction_freeze() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_freeze.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1758589896000,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let tx = result.unwrap();
        assert_eq!(tx.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(tx.value, "100000000");
        assert_eq!(tx.from, tx.to);
    }

    #[test]
    fn test_map_transaction_stake() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_stake.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1758225849000,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let tx = result.unwrap();
        assert_eq!(tx.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(tx.value, "0");
        assert_eq!(tx.from, tx.to);
    }

    #[test]
    fn test_map_transaction_unfreeze() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_unfreeze.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1758596982000,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let tx = result.unwrap();
        assert_eq!(tx.transaction_type, TransactionType::StakeUndelegate);
        assert_eq!(tx.value, "100000000");
        assert_eq!(tx.from, tx.to);
    }

    #[test]
    fn test_map_transaction_coin_transfer() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_coin_transfer.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1757976717000,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let tx = result.unwrap();
        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert_eq!(tx.value, "25000000");
        assert_ne!(tx.from, tx.to);
    }

    #[test]
    fn test_map_transaction_token_transfer() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_token_transfer.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1727747910000,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: Some(vec![crate::models::TronLog {
                topics: Some(vec![
                    "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(),
                    "0000000000000000000000002e1d447fa4169390cf5f5b3d12d380decfbfe20f".to_string(),
                    "0000000000000000000000006e2cf2878020b966786f01ab45ea1fcef6880092".to_string(),
                ]),
                data: Some("00000000000000000000000000000000000000000000000000000000017d7840".to_string()),
            }]),
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let tx = result.unwrap();
        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert_ne!(tx.from, tx.to);
    }
}
