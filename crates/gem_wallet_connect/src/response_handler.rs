use crate::actions::WalletConnectResponseType;
use primitives::ChainType;

pub struct WalletConnectResponseHandler;

impl WalletConnectResponseHandler {
    pub fn encode_sign_message(chain_type: ChainType, signature: String) -> WalletConnectResponseType {
        match chain_type {
            ChainType::Solana | ChainType::Sui | ChainType::Tron => {
                let result = serde_json::json!({
                    "signature": signature
                });
                WalletConnectResponseType::Object {
                    json: serde_json::to_string(&result).unwrap_or_default(),
                }
            }
            ChainType::Ton | ChainType::Bitcoin => WalletConnectResponseType::Object { json: signature },
            _ => WalletConnectResponseType::String { value: signature },
        }
    }

    pub fn encode_sign_transaction(chain_type: ChainType, transaction_id: String) -> WalletConnectResponseType {
        match chain_type {
            ChainType::Solana | ChainType::Ton => WalletConnectResponseType::Object {
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
                WalletConnectResponseType::Object { json: result.to_string() }
            }
            ChainType::Tron => WalletConnectResponseType::Object { json: transaction_id },
            _ => WalletConnectResponseType::String { value: transaction_id },
        }
    }

    pub fn encode_sign_all_transactions(signed_transactions: Vec<String>) -> WalletConnectResponseType {
        WalletConnectResponseType::Object {
            json: serde_json::json!({ "transactions": signed_transactions }).to_string(),
        }
    }

    pub fn encode_send_transaction(chain_type: ChainType, transaction_id: String) -> WalletConnectResponseType {
        match chain_type {
            ChainType::Sui => WalletConnectResponseType::Object {
                json: serde_json::json!({ "digest": transaction_id }).to_string(),
            },
            ChainType::Tron => WalletConnectResponseType::Object {
                json: serde_json::json!({ "result": true, "txid": transaction_id }).to_string(),
            },
            // TON broadcasts and returns the signed BOC, not the broadcast hash.
            ChainType::Ton => WalletConnectResponseType::String { value: transaction_id },
            _ => WalletConnectResponseType::String { value: transaction_id },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(json: &str) -> WalletConnectResponseType {
        WalletConnectResponseType::Object { json: json.to_string() }
    }

    fn string(value: &str) -> WalletConnectResponseType {
        WalletConnectResponseType::String { value: value.to_string() }
    }

    #[test]
    fn test_encode_sign_message_ethereum() {
        assert_eq!(WalletConnectResponseHandler::encode_sign_message(ChainType::Ethereum, "0xsig".to_string()), string("0xsig"));
    }

    #[test]
    fn test_encode_sign_message_solana() {
        assert_eq!(
            WalletConnectResponseHandler::encode_sign_message(ChainType::Solana, "sig123".to_string()),
            object(r#"{"signature":"sig123"}"#)
        );
    }

    #[test]
    fn test_encode_sign_transaction_tron() {
        assert_eq!(
            WalletConnectResponseHandler::encode_sign_transaction(ChainType::Tron, r#"{"signature":["sig"]}"#.to_string()),
            object(r#"{"signature":["sig"]}"#)
        );
    }

    #[test]
    fn test_encode_sign_transaction_sui() {
        assert_eq!(
            WalletConnectResponseHandler::encode_sign_transaction(ChainType::Sui, "txbytes_sig123".to_string()),
            object(r#"{"signature":"sig123","transactionBytes":"txbytes"}"#)
        );
    }

    #[test]
    fn test_encode_send_transaction_sui() {
        assert_eq!(
            WalletConnectResponseHandler::encode_send_transaction(ChainType::Sui, "digest123".to_string()),
            object(r#"{"digest":"digest123"}"#)
        );
    }

    #[test]
    fn test_encode_send_transaction_ton() {
        assert_eq!(
            WalletConnectResponseHandler::encode_send_transaction(ChainType::Ton, "te6ccg...".to_string()),
            string("te6ccg...")
        );
    }

    #[test]
    fn test_encode_send_transaction_tron() {
        assert_eq!(
            WalletConnectResponseHandler::encode_send_transaction(ChainType::Tron, "txid123".to_string()),
            object(r#"{"result":true,"txid":"txid123"}"#)
        );
    }

    #[test]
    fn test_encode_sign_all_transactions() {
        assert_eq!(
            WalletConnectResponseHandler::encode_sign_all_transactions(vec!["signed_tx_1".to_string(), "signed_tx_2".to_string()]),
            object(r#"{"transactions":["signed_tx_1","signed_tx_2"]}"#)
        );
    }
}
