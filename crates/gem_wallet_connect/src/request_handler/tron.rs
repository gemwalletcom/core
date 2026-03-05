use crate::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::sign_type::SignDigestType;
use primitives::{Chain, TransferDataOutputType, ValueAccess};
use serde_json::Value;

pub struct TronRequestHandler;

impl TronRequestHandler {
    pub fn parse_sign_message(_chain: Chain, params: Value, _domain: &str) -> Result<WalletConnectAction, String> {
        let message = params.get_value("message")?.string()?.to_string();

        Ok(WalletConnectAction::SignMessage {
            chain: Chain::Tron,
            sign_type: SignDigestType::TronPersonal,
            data: message,
        })
    }

    pub fn parse_sign_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_value("transaction")?;

        Ok(WalletConnectAction::SignTransaction {
            chain: Chain::Tron,
            transaction_type: WalletConnectTransactionType::Tron {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data: params.to_string(),
        })
    }

    pub fn parse_send_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_value("transaction")?;

        Ok(WalletConnectAction::SendTransaction {
            chain: Chain::Tron,
            transaction_type: WalletConnectTransactionType::Tron {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data: params.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sign_message() {
        let params = serde_json::from_str(r#"{"message":"Hello"}"#).unwrap();
        assert_eq!(
            TronRequestHandler::parse_sign_message(Chain::Tron, params, "example.com").unwrap(),
            WalletConnectAction::SignMessage {
                chain: Chain::Tron,
                sign_type: SignDigestType::TronPersonal,
                data: "Hello".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sign_transaction() {
        let params: Value = serde_json::from_str(r#"{"transaction":{"raw_data_hex":"abc"}}"#).unwrap();
        let expected_data = params.to_string();
        assert_eq!(
            TronRequestHandler::parse_sign_transaction(Chain::Tron, params).unwrap(),
            WalletConnectAction::SignTransaction {
                chain: Chain::Tron,
                transaction_type: WalletConnectTransactionType::Tron {
                    output_type: TransferDataOutputType::EncodedTransaction,
                },
                data: expected_data,
            }
        );
    }

    #[test]
    fn test_parse_send_transaction() {
        let params: Value = serde_json::from_str(r#"{"transaction":{"raw_data_hex":"abc"}}"#).unwrap();
        let expected_data = params.to_string();
        assert_eq!(
            TronRequestHandler::parse_send_transaction(Chain::Tron, params).unwrap(),
            WalletConnectAction::SendTransaction {
                chain: Chain::Tron,
                transaction_type: WalletConnectTransactionType::Tron {
                    output_type: TransferDataOutputType::EncodedTransaction,
                },
                data: expected_data,
            }
        );
    }

    #[test]
    fn test_parse_send_transaction_with_testdata() {
        use crate::WalletConnectRequestHandler;
        use primitives::WalletConnectRequest;

        let params = include_str!("../../testdata/tron_send_transaction.json");
        let expected_data: serde_json::Value = serde_json::from_str(params.trim()).unwrap();
        let expected_data = expected_data.to_string();
        let request = WalletConnectRequest::mock("tron_sendTransaction", &serde_json::to_string(&params.trim()).unwrap(), Some("tron:0x2b6653dc"));

        let action = WalletConnectRequestHandler::parse_request(request).unwrap();
        assert_eq!(
            action,
            WalletConnectAction::SendTransaction {
                chain: Chain::Tron,
                transaction_type: WalletConnectTransactionType::Tron {
                    output_type: TransferDataOutputType::EncodedTransaction,
                },
                data: expected_data,
            }
        );
    }
}
