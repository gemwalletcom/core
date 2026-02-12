use crate::message::sign_type::SignDigestType;
use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::wallet_connect::handler_traits::ChainRequestHandler;
use primitives::{Chain, TransferDataOutputType, ValueAccess};
use serde_json::Value;

// https://docs.reown.com/advanced/multichain/rpc-reference/tron-rpc
pub struct TronRequestHandler;

impl ChainRequestHandler for TronRequestHandler {
    fn parse_sign_message(_chain: Chain, params: Value, _domain: &str) -> Result<WalletConnectAction, String> {
        let message = params.get_value("message")?.string()?.to_string();

        Ok(WalletConnectAction::SignMessage {
            chain: Chain::Tron,
            sign_type: SignDigestType::TronPersonal,
            data: message,
        })
    }

    fn parse_sign_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_value("transaction")?;

        Ok(WalletConnectAction::SignTransaction {
            chain: Chain::Tron,
            transaction_type: WalletConnectTransactionType::Tron {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data: params.to_string(),
        })
    }

    fn parse_send_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
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
        let params: serde_json::Value = serde_json::from_str(r#"{"transaction":{"raw_data_hex":"abc"}}"#).unwrap();
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
        let params: serde_json::Value = serde_json::from_str(r#"{"transaction":{"raw_data_hex":"abc"}}"#).unwrap();
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
}
