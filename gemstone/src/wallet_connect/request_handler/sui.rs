use crate::message::sign_type::SignDigestType;
use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::wallet_connect::handler_traits::ChainRequestHandler;
use primitives::{Chain, TransferDataOutputType, ValueAccess};
use serde_json::Value;

pub struct SuiRequestHandler;

impl ChainRequestHandler for SuiRequestHandler {
    fn parse_sign_message(_chain: Chain, params: Value, _domain: &str) -> Result<WalletConnectAction, String> {
        let message = params.get_value("message")?.string()?.to_string();

        Ok(WalletConnectAction::SignMessage {
            chain: Chain::Sui,
            sign_type: SignDigestType::SuiPersonal,
            data: message,
        })
    }

    fn parse_sign_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_value("transaction")?.string()?;

        Ok(WalletConnectAction::SignTransaction {
            chain: Chain::Sui,
            transaction_type: WalletConnectTransactionType::Sui {
                output_type: TransferDataOutputType::Signature,
            },
            data: params.to_string(),
        })
    }

    fn parse_send_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_value("transaction")?.string()?;

        Ok(WalletConnectAction::SendTransaction {
            chain: Chain::Sui,
            transaction_type: WalletConnectTransactionType::Sui {
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
        let params = serde_json::from_str(r#"{"message":"Hello Sui"}"#).unwrap();
        assert_eq!(
            SuiRequestHandler::parse_sign_message(Chain::Sui, params, "example.com").unwrap(),
            WalletConnectAction::SignMessage {
                chain: Chain::Sui,
                sign_type: SignDigestType::SuiPersonal,
                data: "Hello Sui".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sign_transaction() {
        let params: serde_json::Value =
            serde_json::from_str(r#"{"address":"0xfa92fe9555eeb34d3d922dae643483cbd18bd607bf900a1df5e82dc22804698e","transaction":"AAACAAhkAAA"}"#).unwrap();
        let expected_data = params.to_string();
        assert_eq!(
            SuiRequestHandler::parse_sign_transaction(Chain::Sui, params).unwrap(),
            WalletConnectAction::SignTransaction {
                chain: Chain::Sui,
                transaction_type: WalletConnectTransactionType::Sui {
                    output_type: TransferDataOutputType::Signature,
                },
                data: expected_data,
            }
        );
    }

    #[test]
    fn test_parse_send_transaction() {
        let params: serde_json::Value =
            serde_json::from_str(r#"{"address":"0xfa92fe9555eeb34d3d922dae643483cbd18bd607bf900a1df5e82dc22804698e","transaction":"AAACAAhkAAA"}"#).unwrap();
        let expected_data = params.to_string();
        assert_eq!(
            SuiRequestHandler::parse_send_transaction(Chain::Sui, params).unwrap(),
            WalletConnectAction::SendTransaction {
                chain: Chain::Sui,
                transaction_type: WalletConnectTransactionType::Sui {
                    output_type: TransferDataOutputType::EncodedTransaction,
                },
                data: expected_data,
            }
        );
    }
}
