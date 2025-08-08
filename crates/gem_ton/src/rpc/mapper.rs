use chrono::DateTime;
use primitives::{chain::Chain, Transaction, TransactionState, TransactionType};
use tonlib_core::TonAddress;

use super::model::Transaction as TonTransaction;

pub struct TonMapper;

impl TonMapper {
    pub fn parse_address(address: &str) -> Option<String> {
        Some(TonAddress::from_hex_str(address).ok()?.to_base64_url())
    }

    pub fn map_transactions(chain: Chain, transactions: Vec<TonTransaction>) -> Vec<Transaction> {
        transactions
            .into_iter()
            .flat_map(|x| Self::map_transaction(chain, x))
            .collect::<Vec<Transaction>>()
    }

    pub fn map_transaction(chain: Chain, transaction: TonTransaction) -> Option<Transaction> {
        if transaction.transaction_type != "TransOrd" {
            return None;
        }

        let asset_id = chain.as_asset_id();
        let state = if transaction.success {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };
        let created_at = DateTime::from_timestamp(transaction.utime, 0)?;
        let in_msg = transaction.in_msg.as_ref()?;
        let hash = in_msg.hash.clone();

        // Handle outgoing transfers (with out messages)
        if transaction.out_msgs.len() == 1 && transaction.out_msgs.first()?.op_code.is_none() {
            let out_message = transaction.out_msgs.first()?;
            let from = Self::parse_address(&out_message.source.address)?;
            let to = match &out_message.destination {
                Some(destination) => Self::parse_address(&destination.address)?,
                None => return None,
            };
            let value = out_message.value.to_string();

            return Some(Transaction::new(
                hash,
                asset_id.clone(),
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                transaction.total_fees.to_string(),
                asset_id,
                value,
                None, // memo
                None,
                created_at,
            ));
        }

        // Handle incoming transfers (with in message but no out messages)
        if transaction.out_msgs.is_empty() {
            // Check if this is an internal message with value transfer
            if let (Some(msg_type), Some(value), Some(source), Some(destination)) = (&in_msg.msg_type, in_msg.value, &in_msg.source, &in_msg.destination) {
                if msg_type == "int_msg" && value > 0 {
                    let from = Self::parse_address(&source.address)?;
                    let to = Self::parse_address(&destination.address)?;

                    return Some(Transaction::new(
                        hash,
                        asset_id.clone(),
                        from,
                        to,
                        None,
                        TransactionType::Transfer,
                        state,
                        transaction.total_fees.to_string(),
                        asset_id,
                        value.to_string(),
                        None, // memo
                        None,
                        created_at,
                    ));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;
    use serde_json;

    #[test]
    fn test_map_incoming_transaction() {
        // Real TON incoming transaction JSON (27.013 TON received)
        let json_data = r#"{
            "hash": "0418c1b2e56653421a17f4b18f5964a931a581c6985d097a0e83b2175480de4e",
            "in_msg": {
                "hash": "cd0163d819dc5a0f0b98133bfebe321b45a6f93e7ab8c7bb8bd68afded95d41d",
                "msg_type": "int_msg",
                "value": 27013390900,
                "source": {
                    "address": "0:ca3bc9c2b159350d76d809cd9ec787ca198877005db319e03799376a48ad9be0"
                },
                "destination": {
                    "address": "0:6996055eb4b51d04ed733ef08387cb6b57347e44f386a0ad6c434b57ca535f57"
                }
            },
            "block": "(0,8000000000000000,55559979)",
            "transaction_type": "TransOrd",
            "total_fees": 310015,
            "out_msgs": [],
            "success": true,
            "utime": 1753846990
        }"#;

        let ton_tx: crate::rpc::model::Transaction = serde_json::from_str(json_data).unwrap();
        let result = TonMapper::map_transaction(Chain::Ton, ton_tx);

        assert!(result.is_some());
        let transaction = result.unwrap();

        // Verify transaction details
        assert_eq!(transaction.hash, "cd0163d819dc5a0f0b98133bfebe321b45a6f93e7ab8c7bb8bd68afded95d41d");
        assert_eq!(transaction.asset_id, Chain::Ton.as_asset_id());
        assert_eq!(transaction.from, "EQDKO8nCsVk1DXbYCc2ex4fKGYh3AF2zGeA3mTdqSK2b4BXX");
        assert_eq!(transaction.to, "EQBplgVetLUdBO1zPvCDh8trVzR-RPOGoK1sQ0tXylNfV-Yn");
        assert_eq!(transaction.value, "27013390900");
        assert_eq!(transaction.fee, "310015");
        assert_eq!(transaction.fee_asset_id, Chain::Ton.as_asset_id());
        assert_eq!(transaction.transaction_type, primitives::TransactionType::Transfer);
        assert_eq!(transaction.state, primitives::TransactionState::Confirmed);
    }

    #[test]
    fn test_map_outgoing_transaction() {
        // Real TON outgoing transaction JSON (1.0 TON sent)
        let json_data = r#"{
            "hash": "ac0b4e14b0db64226c1938e5923216190150970f47094a55ea680f3d1a2be12f",
            "in_msg": {
                "hash": "dcf57aec3405981dbd1c46729e6d56730770d37fd081b51d150c1ed03f8eb4a2",
                "msg_type": "ext_in_msg",
                "value": 0,
                "source": null,
                "destination": {
                    "address": "0:6996055eb4b51d04ed733ef08387cb6b57347e44f386a0ad6c434b57ca535f57"
                }
            },
            "block": "(0,6000000000000000,54323098)",
            "transaction_type": "TransOrd",
            "total_fees": 666693,
            "out_msgs": [
                {
                    "source": {
                        "address": "0:6996055eb4b51d04ed733ef08387cb6b57347e44f386a0ad6c434b57ca535f57"
                    },
                    "destination": {
                        "address": "0:90effeeb58defac39dd1f4028d495c0c70d014f23603196b7587018e8ad59b4d"
                    },
                    "value": 1000000000,
                    "op_code": null,
                    "decoded_op_name": null
                }
            ],
            "success": true,
            "utime": 1750595597
        }"#;

        let ton_tx: crate::rpc::model::Transaction = serde_json::from_str(json_data).unwrap();
        let result = TonMapper::map_transaction(Chain::Ton, ton_tx);

        assert!(result.is_some());
        let transaction = result.unwrap();

        // Verify transaction details
        assert_eq!(transaction.hash, "dcf57aec3405981dbd1c46729e6d56730770d37fd081b51d150c1ed03f8eb4a2");
        assert_eq!(transaction.asset_id, Chain::Ton.as_asset_id());
        assert_eq!(transaction.from, "EQBplgVetLUdBO1zPvCDh8trVzR-RPOGoK1sQ0tXylNfV-Yn");
        assert_eq!(transaction.to, "EQCQ7_7rWN76w53R9AKNSVwMcNAU8jYDGWt1hwGOitWbTVwj");
        assert_eq!(transaction.value, "1000000000");
        assert_eq!(transaction.fee, "666693");
        assert_eq!(transaction.fee_asset_id, Chain::Ton.as_asset_id());
        assert_eq!(transaction.transaction_type, primitives::TransactionType::Transfer);
        assert_eq!(transaction.state, primitives::TransactionState::Confirmed);
    }

    #[test]
    fn test_reject_non_transord_transaction() {
        // Transaction with different type should be rejected
        let json_data = r#"{
            "hash": "test_hash",
            "in_msg": {
                "hash": "test_in_msg_hash"
            },
            "block": "test_block",
            "transaction_type": "TransTick",
            "total_fees": 100000,
            "out_msgs": [],
            "success": true,
            "utime": 1750000000
        }"#;

        let ton_tx: crate::rpc::model::Transaction = serde_json::from_str(json_data).unwrap();
        let result = TonMapper::map_transaction(Chain::Ton, ton_tx);

        assert!(result.is_none());
    }

    #[test]
    fn test_reject_incoming_transaction_with_zero_value() {
        // Incoming transaction with zero value should be rejected
        let json_data = r#"{
            "hash": "test_hash",
            "in_msg": {
                "hash": "test_in_msg_hash",
                "msg_type": "int_msg",
                "value": 0,
                "source": {
                    "address": "0:ca3bc9c2b159350d76d809cd9ec787ca198877005db319e03799376a48ad9be0"
                },
                "destination": {
                    "address": "0:6996055eb4b51d04ed733ef08387cb6b57347e44f386a0ad6c434b57ca535f57"
                }
            },
            "block": "test_block",
            "transaction_type": "TransOrd",
            "total_fees": 100000,
            "out_msgs": [],
            "success": true,
            "utime": 1750000000
        }"#;

        let ton_tx: crate::rpc::model::Transaction = serde_json::from_str(json_data).unwrap();
        let result = TonMapper::map_transaction(Chain::Ton, ton_tx);

        assert!(result.is_none());
    }

    #[test]
    fn test_reject_outgoing_transaction_with_op_code() {
        // Outgoing transaction with op_code should be rejected (contract interaction)
        let json_data = r#"{
            "hash": "test_hash",
            "in_msg": {
                "hash": "test_in_msg_hash"
            },
            "block": "test_block",
            "transaction_type": "TransOrd",
            "total_fees": 100000,
            "out_msgs": [
                {
                    "source": {
                        "address": "0:6996055eb4b51d04ed733ef08387cb6b57347e44f386a0ad6c434b57ca535f57"
                    },
                    "destination": {
                        "address": "0:90effeeb58defac39dd1f4028d495c0c70d014f23603196b7587018e8ad59b4d"
                    },
                    "value": 1000000000,
                    "op_code": "0x12345678",
                    "decoded_op_name": "transfer"
                }
            ],
            "success": true,
            "utime": 1750000000
        }"#;

        let ton_tx: crate::rpc::model::Transaction = serde_json::from_str(json_data).unwrap();
        let result = TonMapper::map_transaction(Chain::Ton, ton_tx);

        assert!(result.is_none());
    }

    #[test]
    fn test_reject_outgoing_transaction_without_destination() {
        // Outgoing transaction without destination should be rejected (malformed data)
        let json_data = r#"{
            "hash": "test_hash",
            "in_msg": {
                "hash": "test_in_msg_hash"
            },
            "block": "test_block",
            "transaction_type": "TransOrd",
            "total_fees": 100000,
            "out_msgs": [
                {
                    "source": {
                        "address": "0:6996055eb4b51d04ed733ef08387cb6b57347e44f386a0ad6c434b57ca535f57"
                    },
                    "destination": null,
                    "value": 1000000000,
                    "op_code": null,
                    "decoded_op_name": null
                }
            ],
            "success": true,
            "utime": 1750000000
        }"#;

        let ton_tx: crate::rpc::model::Transaction = serde_json::from_str(json_data).unwrap();
        let result = TonMapper::map_transaction(Chain::Ton, ton_tx);

        assert!(result.is_none());
    }

    #[test]
    fn test_map_failed_transaction() {
        // Failed incoming transaction should still be parsed with Failed state
        let json_data = r#"{
            "hash": "failed_tx_hash",
            "in_msg": {
                "hash": "failed_in_msg_hash",
                "msg_type": "int_msg",
                "value": 1000000000,
                "source": {
                    "address": "0:ca3bc9c2b159350d76d809cd9ec787ca198877005db319e03799376a48ad9be0"
                },
                "destination": {
                    "address": "0:6996055eb4b51d04ed733ef08387cb6b57347e44f386a0ad6c434b57ca535f57"
                }
            },
            "block": "test_block",
            "transaction_type": "TransOrd",
            "total_fees": 100000,
            "out_msgs": [],
            "success": false,
            "utime": 1750000000
        }"#;

        let ton_tx: crate::rpc::model::Transaction = serde_json::from_str(json_data).unwrap();
        let result = TonMapper::map_transaction(Chain::Ton, ton_tx);

        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.state, primitives::TransactionState::Failed);
    }
}
