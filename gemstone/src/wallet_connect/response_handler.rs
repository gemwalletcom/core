use primitives::ChainType;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum WalletConnectResponseType {
    String { value: String },
    Object { json: String },
}

pub struct WalletConnectResponseHandler;

impl WalletConnectResponseHandler {
    pub fn encode_sign_message(chain_type: ChainType, signature: String) -> WalletConnectResponseType {
        match chain_type {
            ChainType::Solana | ChainType::Sui => {
                let result = serde_json::json!({
                    "signature": signature
                });
                WalletConnectResponseType::Object {
                    json: serde_json::to_string(&result).unwrap_or_default(),
                }
            }
            _ => WalletConnectResponseType::String { value: signature },
        }
    }

    pub fn encode_sign_transaction(chain_type: ChainType, transaction_id: String) -> WalletConnectResponseType {
        match chain_type {
            ChainType::Solana => WalletConnectResponseType::Object {
                json: serde_json::json!({ "signature": transaction_id }).to_string(),
            },
            ChainType::Sui => {
                let parts: Vec<&str> = transaction_id.splitn(2, '_').collect();
                let result = if parts.len() == 2 {
                    serde_json::json!({
                        "signature": parts[1],
                        "transactionBytes": parts[0]
                    })
                } else {
                    serde_json::json!({
                        "signature": transaction_id,
                        "transactionBytes": ""
                    })
                };
                WalletConnectResponseType::Object {
                    json: result.to_string(),
                }
            }
            _ => WalletConnectResponseType::String { value: transaction_id },
        }
    }

    pub fn encode_send_transaction(chain_type: ChainType, transaction_id: String) -> WalletConnectResponseType {
        match chain_type {
            ChainType::Sui => WalletConnectResponseType::Object {
                json: serde_json::json!({ "digest": transaction_id }).to_string(),
            },
            _ => WalletConnectResponseType::String { value: transaction_id },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_sign_message_ethereum() {
        let result = WalletConnectResponseHandler::encode_sign_message(ChainType::Ethereum, "0xsignature".to_string());
        assert!(matches!(result, WalletConnectResponseType::String { value } if value == "0xsignature"));
    }

    #[test]
    fn test_encode_sign_message_solana() {
        let result = WalletConnectResponseHandler::encode_sign_message(ChainType::Solana, "signature123".to_string());
        match result {
            WalletConnectResponseType::Object { json } => {
                assert!(json.contains("\"signature\""));
                assert!(json.contains("signature123"));
            }
            _ => panic!("Expected Object response for Solana"),
        }
    }

    #[test]
    fn test_encode_sign_message_sui() {
        let result = WalletConnectResponseHandler::encode_sign_message(ChainType::Sui, "suisig123".to_string());
        match result {
            WalletConnectResponseType::Object { json } => {
                assert!(json.contains("\"signature\""));
                assert!(json.contains("suisig123"));
            }
            _ => panic!("Expected Object response for Sui"),
        }
    }

    #[test]
    fn test_encode_sign_transaction_ethereum() {
        let result = WalletConnectResponseHandler::encode_sign_transaction(ChainType::Ethereum, "0xtxid".to_string());
        assert!(matches!(result, WalletConnectResponseType::String { value } if value == "0xtxid"));
    }

    #[test]
    fn test_encode_sign_transaction_solana() {
        let result = WalletConnectResponseHandler::encode_sign_transaction(ChainType::Solana, "txid123".to_string());
        match result {
            WalletConnectResponseType::Object { json } => {
                assert!(json.contains("\"signature\""));
                assert!(json.contains("txid123"));
            }
            _ => panic!("Expected Object response for Solana"),
        }
    }

    #[test]
    fn test_encode_sign_transaction_sui() {
        let result = WalletConnectResponseHandler::encode_sign_transaction(ChainType::Sui, "txbytes_sig123".to_string());
        match result {
            WalletConnectResponseType::Object { json } => {
                assert!(json.contains("\"signature\""));
                assert!(json.contains("\"transactionBytes\""));
                assert!(json.contains("sig123"));
                assert!(json.contains("txbytes"));
            }
            _ => panic!("Expected Object response for Sui"),
        }
    }

    #[test]
    fn test_encode_send_transaction_ethereum() {
        let result = WalletConnectResponseHandler::encode_send_transaction(ChainType::Ethereum, "0xhash".to_string());
        assert!(matches!(result, WalletConnectResponseType::String { value } if value == "0xhash"));
    }

    #[test]
    fn test_encode_send_transaction_sui() {
        let result = WalletConnectResponseHandler::encode_send_transaction(ChainType::Sui, "digest123".to_string());
        match result {
            WalletConnectResponseType::Object { json } => {
                assert!(json.contains("\"digest\""));
                assert!(json.contains("digest123"));
            }
            _ => panic!("Expected Object response for Sui"),
        }
    }
}
