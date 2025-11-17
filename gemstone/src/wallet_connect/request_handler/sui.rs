use crate::message::sign_type::SignDigestType;
use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectTransactionType};
use primitives::{Chain, TransferDataOutputType};
use serde_json::Value;

pub struct SuiRequestHandler;

impl SuiRequestHandler {
    pub fn parse_sign_message(params: Value) -> Result<WalletConnectAction, String> {
        let message = params.get("message").and_then(|v| v.as_str()).ok_or("Missing message parameter")?;

        Ok(WalletConnectAction::SignMessage {
            chain: Chain::Sui,
            sign_type: SignDigestType::Sign,
            data: message.to_string(),
        })
    }

    pub fn parse_sign_transaction(params: Value) -> Result<WalletConnectAction, String> {
        params.get("transaction").and_then(|v| v.as_str()).ok_or("Missing transaction parameter")?;

        let data = serde_json::to_string(&params).map_err(|e| format!("Failed to serialize params: {}", e))?;

        Ok(WalletConnectAction::SignTransaction {
            chain: Chain::Sui,
            transaction_type: WalletConnectTransactionType::Sui {
                output_type: TransferDataOutputType::Signature,
            },
            data,
        })
    }

    pub fn parse_send_transaction(params: Value) -> Result<WalletConnectAction, String> {
        params.get("transaction").and_then(|v| v.as_str()).ok_or("Missing transaction parameter")?;

        let data = serde_json::to_string(&params).map_err(|e| format!("Failed to serialize params: {}", e))?;

        Ok(WalletConnectAction::SendTransaction {
            chain: Chain::Sui,
            transaction_type: WalletConnectTransactionType::Sui {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sign_message() {
        let params = serde_json::from_str(r#"{"message":"Hello Sui"}"#).unwrap();
        let action = SuiRequestHandler::parse_sign_message(params).unwrap();
        match action {
            WalletConnectAction::SignMessage { chain, sign_type, data } => {
                assert_eq!(chain, Chain::Sui);
                assert!(matches!(sign_type, SignDigestType::Sign));
                assert_eq!(data, "Hello Sui");
            }
            _ => panic!("Expected SignMessage action"),
        }
    }

    #[test]
    fn test_parse_sign_transaction() {
        let params =
            serde_json::from_str(r#"{"address":"0xfa92fe9555eeb34d3d922dae643483cbd18bd607bf900a1df5e82dc22804698e","transaction":"AAACAAhkAAA"}"#).unwrap();
        let action = SuiRequestHandler::parse_sign_transaction(params).unwrap();
        match action {
            WalletConnectAction::SignTransaction { chain, transaction_type, data } => {
                assert_eq!(chain, Chain::Sui);
                assert!(matches!(transaction_type, WalletConnectTransactionType::Sui { .. }));
                assert!(data.contains("\"address\""));
                assert!(data.contains("\"transaction\""));
                assert!(data.contains("0xfa92fe9555eeb34d3d922dae643483cbd18bd607bf900a1df5e82dc22804698e"));
            }
            _ => panic!("Expected SignTransaction action"),
        }
    }

    #[test]
    fn test_parse_send_transaction() {
        let params =
            serde_json::from_str(r#"{"address":"0xfa92fe9555eeb34d3d922dae643483cbd18bd607bf900a1df5e82dc22804698e","transaction":"AAACAAhkAAA"}"#).unwrap();
        let action = SuiRequestHandler::parse_send_transaction(params).unwrap();
        match action {
            WalletConnectAction::SendTransaction { chain, transaction_type, data } => {
                assert_eq!(chain, Chain::Sui);
                assert!(matches!(transaction_type, WalletConnectTransactionType::Sui { .. }));
                assert!(data.contains("\"address\""));
                assert!(data.contains("\"transaction\""));
                assert!(data.contains("0xfa92fe9555eeb34d3d922dae643483cbd18bd607bf900a1df5e82dc22804698e"));
            }
            _ => panic!("Expected SendTransaction action"),
        }
    }
}
