use chrono::{DateTime, Utc};
use primitives::{chain::Chain, Transaction, TransactionState, TransactionType, TransactionUpdate};

use crate::constants::{TRANSACTION_TYPE_TRANSFER_ALLOW_DEATH, TRANSACTION_TYPE_TRANSFER_KEEP_ALIVE};
use crate::models::rpc::{Block, Extrinsic, ExtrinsicArguments};

pub fn map_transactions(chain: Chain, block: Block) -> Vec<Transaction> {
    let first = block.extrinsics.first();
    let created_at = match first {
        Some(Extrinsic {
            args: ExtrinsicArguments::Timestamp(timestamp),
            ..
        }) => DateTime::from_timestamp_millis(timestamp.now as i64),
        _ => None,
    }
    .expect("Timestamp not found");

    block
        .extrinsics
        .iter()
        .flat_map(|x| map_transaction(chain, x.clone(), created_at))
        .flatten()
        .collect()
}

pub fn map_transaction(chain: Chain, transaction: Extrinsic, created_at: DateTime<Utc>) -> Vec<Option<Transaction>> {
    match &transaction.args.clone() {
        ExtrinsicArguments::Transfer(transfer) => {
            vec![map_transfer(
                chain,
                transaction.clone(),
                transaction.method.method.clone(),
                transfer.dest.id.clone(),
                transfer.value.clone(),
                created_at,
            )]
        }
        ExtrinsicArguments::Transfers(transfers) => transfers
            .calls
            .iter()
            .map(|x| {
                map_transfer(
                    chain,
                    transaction.clone(),
                    x.method.method.clone(),
                    x.args.dest.id.clone(),
                    x.args.value.clone(),
                    created_at,
                )
            })
            .collect(),
        _ => vec![],
    }
}

fn map_transfer(chain: Chain, transaction: Extrinsic, method: String, to_address: String, value: String, created_at: DateTime<Utc>) -> Option<Transaction> {
    if method != TRANSACTION_TYPE_TRANSFER_ALLOW_DEATH && method != TRANSACTION_TYPE_TRANSFER_KEEP_ALIVE {
        return None;
    }

    let from_address = transaction.signature?.signer.id.clone();
    let state = if transaction.success {
        TransactionState::Confirmed
    } else {
        TransactionState::Failed
    };

    Some(Transaction::new(
        transaction.hash.clone(),
        chain.as_asset_id(),
        from_address,
        to_address,
        None,
        TransactionType::Transfer,
        state,
        transaction.info.partial_fee.unwrap_or("0".to_string()),
        chain.as_asset_id(),
        value,
        None,
        None,
        created_at,
    ))
}

pub fn map_transaction_status(blocks: Vec<Block>, transaction_id: &str, block_number: i64) -> TransactionUpdate {
    for block in blocks {
        for extrinsic in block.extrinsics {
            if extrinsic.hash == transaction_id {
                let state = if extrinsic.success {
                    TransactionState::Confirmed
                } else {
                    TransactionState::Failed
                };
                return TransactionUpdate::new_state(state);
            }
        }
    }

    TransactionUpdate::new(
        TransactionState::Pending,
        vec![primitives::TransactionChange::BlockNumber(block_number.to_string())],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rpc::{Extrinsic, ExtrinsicArguments, ExtrinsicInfo, ExtrinsicMethod};

    fn create_test_extrinsic(hash: &str, success: bool) -> Extrinsic {
        Extrinsic {
            hash: hash.to_string(),
            method: ExtrinsicMethod {
                pallet: "test".to_string(),
                method: "test".to_string(),
            },
            info: ExtrinsicInfo {
                partial_fee: Some("0".to_string()),
            },
            success,
            args: ExtrinsicArguments::Other(serde_json::json!({})),
            signature: None,
        }
    }

    #[test]
    fn test_map_transaction_status_confirmed() {
        let blocks = vec![Block {
            number: 100,
            extrinsics: vec![create_test_extrinsic("hash123", true)],
        }];

        let result = map_transaction_status(blocks, "hash123", 100);
        assert_eq!(result.state, TransactionState::Confirmed);
    }

    #[test]
    fn test_map_transaction_status_failed() {
        let blocks = vec![Block {
            number: 100,
            extrinsics: vec![create_test_extrinsic("hash123", false)],
        }];

        let result = map_transaction_status(blocks, "hash123", 100);
        assert_eq!(result.state, TransactionState::Failed);
    }

    #[test]
    fn test_map_transaction_status_pending() {
        let blocks = vec![Block {
            number: 100,
            extrinsics: vec![create_test_extrinsic("other_hash", true)],
        }];

        let result = map_transaction_status(blocks, "hash123", 100);
        assert_eq!(result.state, TransactionState::Pending);
    }
}
