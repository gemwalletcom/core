use crate::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::sign_type::SignDigestType;
use primitives::{Chain, TransferDataOutputType, ValueAccess};
use serde_json::Value;

pub struct SolanaRequestHandler;

impl SolanaRequestHandler {
    pub fn parse_sign_message(_chain: Chain, params: Value, _domain: &str) -> Result<WalletConnectAction, String> {
        let message = params.get_value("message")?.string()?.to_string();

        Ok(WalletConnectAction::SignMessage {
            chain: Chain::Solana,
            sign_type: SignDigestType::Base58,
            data: message,
        })
    }

    pub fn parse_sign_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_value("transaction")?.string()?;

        Ok(WalletConnectAction::SignTransaction {
            chain: Chain::Solana,
            transaction_type: WalletConnectTransactionType::Solana {
                output_type: TransferDataOutputType::Signature,
            },
            data: params.to_string(),
        })
    }

    pub fn parse_send_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_value("transaction")?.string()?;

        Ok(WalletConnectAction::SendTransaction {
            chain: Chain::Solana,
            transaction_type: WalletConnectTransactionType::Solana {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data: params.to_string(),
        })
    }

    pub fn parse_sign_all_transactions(params: Value) -> Result<WalletConnectAction, String> {
        let transaction = params.get_value("transactions")?.at(0)?.string()?.to_string();

        Ok(WalletConnectAction::SignTransaction {
            chain: Chain::Solana,
            transaction_type: WalletConnectTransactionType::Solana {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data: transaction,
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
            SolanaRequestHandler::parse_sign_message(Chain::Solana, params, "example.com").unwrap(),
            WalletConnectAction::SignMessage {
                chain: Chain::Solana,
                sign_type: SignDigestType::Base58,
                data: "Hello".to_string(),
            }
        );
    }

    #[test]
    fn test_sign_transaction() {
        let params: Value = serde_json::from_str(r#"{"transaction":"AAACAAhkAAA"}"#).unwrap();
        let expected_data = params.to_string();
        assert_eq!(
            SolanaRequestHandler::parse_sign_transaction(Chain::Solana, params).unwrap(),
            WalletConnectAction::SignTransaction {
                chain: Chain::Solana,
                transaction_type: WalletConnectTransactionType::Solana {
                    output_type: TransferDataOutputType::Signature,
                },
                data: expected_data,
            }
        );
    }
}
