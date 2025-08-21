use crate::rpc::model::Block;
use primitives::{TransactionState, TransactionUpdate};

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
    use crate::rpc::model::{Extrinsic, ExtrinsicArguments, ExtrinsicInfo, ExtrinsicMethod};

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