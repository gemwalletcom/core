use crate::wallet_connect::handler_traits::ChainResponseHandler;
use primitives::ChainType;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum WalletConnectResponseType {
    String { value: String },
    Object { json: String },
}

pub struct WalletConnectResponseHandler;

impl ChainResponseHandler for WalletConnectResponseHandler {
    fn encode_sign_message(signature: String) -> WalletConnectResponseType {
        WalletConnectResponseType::String { value: signature }
    }

    fn encode_sign_transaction(transaction_id: String) -> WalletConnectResponseType {
        WalletConnectResponseType::String { value: transaction_id }
    }

    fn encode_send_transaction(transaction_id: String) -> WalletConnectResponseType {
        WalletConnectResponseType::String { value: transaction_id }
    }
}

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

    pub fn encode_send_transaction(chain_type: ChainType, transaction_id: String) -> WalletConnectResponseType {
        match chain_type {
            ChainType::Sui => WalletConnectResponseType::Object {
                json: serde_json::json!({ "digest": transaction_id }).to_string(),
            },
            ChainType::Tron => WalletConnectResponseType::Object {
                json: serde_json::json!({ "result": true, "txid": transaction_id }).to_string(),
            },
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
        assert_eq!(
            WalletConnectResponseHandler::encode_sign_message(ChainType::Ethereum, "0xsignature".to_string()),
            string("0xsignature")
        );
    }

    #[test]
    fn test_encode_sign_message_solana() {
        assert_eq!(
            WalletConnectResponseHandler::encode_sign_message(ChainType::Solana, "signature123".to_string()),
            object(r#"{"signature":"signature123"}"#)
        );
    }

    #[test]
    fn test_encode_sign_message_sui() {
        assert_eq!(
            WalletConnectResponseHandler::encode_sign_message(ChainType::Sui, "suisig123".to_string()),
            object(r#"{"signature":"suisig123"}"#)
        );
    }

    #[test]
    fn test_encode_sign_message_tron() {
        assert_eq!(
            WalletConnectResponseHandler::encode_sign_message(ChainType::Tron, "tronsig123".to_string()),
            object(r#"{"signature":"tronsig123"}"#)
        );
    }

    #[test]
    fn test_encode_sign_transaction_ethereum() {
        assert_eq!(
            WalletConnectResponseHandler::encode_sign_transaction(ChainType::Ethereum, "0xtxid".to_string()),
            string("0xtxid")
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
    fn test_encode_sign_transaction_solana() {
        assert_eq!(
            WalletConnectResponseHandler::encode_sign_transaction(ChainType::Solana, "txid123".to_string()),
            object(r#"{"signature":"txid123"}"#)
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
    fn test_encode_send_transaction_ethereum() {
        assert_eq!(
            WalletConnectResponseHandler::encode_send_transaction(ChainType::Ethereum, "0xhash".to_string()),
            string("0xhash")
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
    fn test_encode_send_transaction_tron() {
        assert_eq!(
            WalletConnectResponseHandler::encode_send_transaction(ChainType::Tron, "txid123".to_string()),
            object(r#"{"result":true,"txid":"txid123"}"#)
        );
    }

    #[test]
    fn test_encode_sign_message_ton() {
        let payload = r#"{"signature":"tonsig123","timestamp":1700000000}"#;
        assert_eq!(WalletConnectResponseHandler::encode_sign_message(ChainType::Ton, payload.to_string()), object(payload));
    }

    #[test]
    fn test_encode_sign_transaction_ton() {
        assert_eq!(
            WalletConnectResponseHandler::encode_sign_transaction(ChainType::Ton, "tontxsig".to_string()),
            object(r#"{"signature":"tontxsig"}"#)
        );
    }

    #[test]
    fn test_encode_send_transaction_ton() {
        assert_eq!(
            WalletConnectResponseHandler::encode_send_transaction(ChainType::Ton, "tonhash123".to_string()),
            string("tonhash123")
        );
    }
}
