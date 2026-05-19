use chrono::DateTime;
use num_bigint::BigUint;
use primitives::{
    Address as _, AssetId, Transaction, TransactionResourceTypeMetadata, TransactionState, TransactionType, chain::Chain, decode_hex, hex::decode_hex_utf8, stake_type::Resource,
};
use std::error::Error;

use crate::address::TronAddress;
use crate::models::{BlockTransactions, Transaction as TronTransaction, TransactionReceiptData, TronContractType, TronTransactionBroadcast};
use crate::rpc::constants::ERC20_TRANSFER_EVENT_SIGNATURE;

fn decode_hex_message(hex_str: &str) -> String {
    decode_hex_utf8(hex_str).unwrap_or_else(|| hex_str.to_string())
}

fn resource_type_metadata(resource: Option<String>) -> Option<serde_json::Value> {
    let resource_type = resource.as_deref().and_then(|resource| resource.parse::<Resource>().ok()).unwrap_or(Resource::Bandwidth);
    serde_json::to_value(TransactionResourceTypeMetadata::new(resource_type)).ok()
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
        .zip(receipts)
        .filter_map(|(transaction, receipt)| map_transaction(chain, transaction, receipt))
        .collect()
}

pub fn map_transactions_by_address(transactions: Vec<TronTransaction>, receipts: Vec<TransactionReceiptData>) -> Vec<Transaction> {
    transactions
        .into_iter()
        .zip(receipts)
        .filter_map(|(transaction, receipt)| map_transaction(Chain::Tron, transaction, receipt))
        .collect()
}

pub fn map_transaction(chain: Chain, transaction: TronTransaction, receipt: TransactionReceiptData) -> Option<Transaction> {
    if let (Some(value), Some(contract_result)) = (transaction.raw_data.contract.first().cloned(), transaction.ret.first().cloned()) {
        let state: TransactionState = if contract_result.contract_ret == "SUCCESS" {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };
        let fee = receipt.fee.unwrap_or_default().to_string();
        let created_at = DateTime::from_timestamp_millis(receipt.block_time_stamp)?;

        let memo = transaction.raw_data.data.as_deref().map(decode_hex_message);
        let contract_value = value.parameter.value;
        let from = contract_value.owner_address.unwrap_or_default();

        let contract_type = value.contract_type;
        if let Some((transaction_type, to, amount, metadata)) = match contract_type {
            Some(TronContractType::Transfer) if !transaction.ret.is_empty() => {
                let to = contract_value.to_address.unwrap_or_default();
                Some((TransactionType::Transfer, to, contract_value.amount.unwrap_or_default().to_string(), None))
            }
            Some(TronContractType::FreezeBalanceV2) => Some((
                TransactionType::StakeFreeze,
                from.clone(),
                contract_value.frozen_balance.unwrap_or_default().to_string(),
                resource_type_metadata(contract_value.resource.clone()),
            )),
            Some(TronContractType::UnfreezeBalanceV2) => Some((
                TransactionType::StakeUnfreeze,
                from.clone(),
                contract_value.unfreeze_balance.unwrap_or_default().to_string(),
                resource_type_metadata(contract_value.resource.clone()),
            )),
            Some(TronContractType::VoteWitness) => {
                let votes = contract_value.votes.as_ref()?;
                let vote = votes.first()?;
                let to = TronAddress::from_hex(vote.vote_address.as_str())?.encode();
                let amount = vote.vote_count * 1_000_000;
                Some((TransactionType::StakeDelegate, to, amount.to_string(), None))
            }
            _ => None,
        } {
            let transaction = Transaction::new(
                transaction.transaction_id,
                chain.as_asset_id(),
                from,
                to,
                None,
                transaction_type,
                state,
                fee,
                chain.as_asset_id(),
                amount,
                memo.clone(),
                metadata,
                created_at,
            );
            return Some(transaction);
        }
        let logs = receipt.log.unwrap_or_default();
        if contract_type == Some(TronContractType::TriggerSmart) && logs.len() == 1 {
            let log = logs.first()?;
            let topics = log.topics.as_ref()?;
            if topics.len() != 3 || topics.first()?.as_str() != ERC20_TRANSFER_EVENT_SIGNATURE {
                return None;
            }

            let from_string = format!("41{}", topics[1].chars().skip(24).collect::<String>());
            let to_string = format!("41{}", topics[2].chars().skip(24).collect::<String>());
            let token_id = contract_value.contract_address?;
            let from = TronAddress::from_hex(from_string.as_str())?.encode();
            let to = TronAddress::from_hex(to_string.as_str())?.encode();
            let value = BigUint::from_bytes_be(&decode_hex(log.data.as_deref()?).ok()?);
            let asset_id = AssetId { chain, token_id: Some(token_id) };

            let transaction = Transaction::new(
                transaction.transaction_id,
                asset_id,
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                fee,
                chain.as_asset_id(),
                value.to_string(),
                memo,
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
    use crate::models::{BlockTransactions, TransactionReceipt, TransactionReceiptData, TronContractType, TronTransactionBroadcast};
    use crate::provider::testkit::TEST_TRANSACTION_ID;
    use primitives::asset_constants::TRON_USDT_TOKEN_ID;

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
    fn test_map_transaction_freeze_bandwidth() {
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
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::StakeFreeze);
        assert_eq!(transaction.value, "100000000");
        assert_eq!(transaction.from, transaction.to);
        assert_eq!(transaction.metadata, serde_json::to_value(TransactionResourceTypeMetadata::new(Resource::Bandwidth)).ok());
    }

    #[test]
    fn test_map_transaction_freeze_energy() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_freeze_energy.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1760552376000,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::StakeFreeze);
        assert_eq!(transaction.value, "10000000");
        assert_eq!(transaction.from, transaction.to);
        assert_eq!(transaction.metadata, serde_json::to_value(TransactionResourceTypeMetadata::new(Resource::Energy)).ok());
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
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(transaction.value, "2125000000");
        assert_eq!(transaction.from, "TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb");
        assert_eq!(transaction.to, "TJvaAeFb8Lykt9RQcVyyTFN2iDvGMuyD4M");
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
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::StakeUnfreeze);
        assert_eq!(transaction.value, "100000000");
        assert_eq!(transaction.from, transaction.to);
        assert_eq!(transaction.metadata, serde_json::to_value(TransactionResourceTypeMetadata::new(Resource::Bandwidth)).ok());
    }

    #[test]
    fn test_map_transaction_by_hash() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_coin_transfer.json")).unwrap();
        let receipt: TransactionReceiptData = serde_json::from_str(include_str!("../../testdata/transaction_coin_transfer_receipt.json")).unwrap();

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.value, "25000000");
        assert_ne!(transaction.from, transaction.to);
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
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_ne!(transaction.from, transaction.to);
    }

    #[test]
    fn test_map_transactions_by_block_ignores_unsupported_contract_types() {
        let block: BlockTransactions = serde_json::from_str(include_str!("../../testdata/block_mixed_contract_types.json")).unwrap();
        let receipts: Vec<TransactionReceiptData> = serde_json::from_str(include_str!("../../testdata/block_mixed_contract_types_receipts.json")).unwrap();

        assert_eq!(block.transactions.len(), 5);
        assert_eq!(block.transactions[0].raw_data.contract[0].contract_type, Some(TronContractType::DelegateResource));
        assert_eq!(block.transactions[1].raw_data.contract[0].contract_type, Some(TronContractType::UnDelegateResource));
        assert_eq!(block.transactions[2].raw_data.contract[0].contract_type, Some(TronContractType::TransferAsset));
        assert_eq!(block.transactions[3].raw_data.contract[0].contract_type, None);

        let transactions = map_transactions_by_block(Chain::Tron, block, receipts);

        assert_eq!(transactions.len(), 1);
        let transaction = transactions.first().unwrap();
        assert_eq!(transaction.hash, "10f1e5b04c0dd39f14d4b5ca270899b36ae9c52ac1b9b64b76360c7373cc0893");
        assert_eq!(transaction.asset_id, AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));
        assert_eq!(transaction.from, "TWBPGLwQw2EbqYLLw1DJnTDt2ZQ9yJW1JJ");
        assert_eq!(transaction.to, "TViSMURdt2dda6Pf163UBZoSfbV9hECvvc");
        assert_eq!(transaction.value, "249000000");
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.state, TransactionState::Confirmed);
    }

    #[test]
    fn test_map_transaction_thorchain_swap() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_thorchain_swap.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1771951038000,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
        };

        let transaction = map_transaction(Chain::Tron, transaction, receipt).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.value, "200000000");
        assert_eq!(transaction.memo.as_deref(), Some("=:TRON.USDT:TNAwd1WFe7GHTxovGU9MeT6mi3J4KAZMvP:0/1/0:g1:50"));
    }
}
