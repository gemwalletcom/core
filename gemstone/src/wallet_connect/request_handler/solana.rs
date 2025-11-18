use crate::message::sign_type::SignDigestType;
use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::wallet_connect::handler_traits::ChainRequestHandler;
use primitives::{Chain, TransferDataOutputType};
use serde_json::Value;

pub struct SolanaRequestHandler;

impl ChainRequestHandler for SolanaRequestHandler {
    fn parse_sign_message(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let message = params.get("message").and_then(|v| v.as_str()).ok_or("Missing message parameter")?.to_string();

        Ok(WalletConnectAction::SignMessage {
            chain: Chain::Solana,
            sign_type: SignDigestType::Base58,
            data: message,
        })
    }

    fn parse_sign_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let transaction = params
            .get("transaction")
            .and_then(|v| v.as_str())
            .ok_or("Missing transaction parameter")?
            .to_string();

        Ok(WalletConnectAction::SignTransaction {
            chain: Chain::Solana,
            transaction_type: WalletConnectTransactionType::Solana {
                output_type: TransferDataOutputType::Signature,
            },
            data: transaction,
        })
    }

    fn parse_send_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let transaction = params
            .get("transaction")
            .and_then(|v| v.as_str())
            .ok_or("Missing transaction parameter")?
            .to_string();

        Ok(WalletConnectAction::SendTransaction {
            chain: Chain::Solana,
            transaction_type: WalletConnectTransactionType::Solana {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data: transaction,
        })
    }
}

impl SolanaRequestHandler {
    pub fn parse_sign_all_transactions(params: Value) -> Result<WalletConnectAction, String> {
        let transactions = params.get("transactions").and_then(|v| v.as_array()).ok_or("Missing transactions parameter")?;
        let transaction = transactions.first().and_then(|v| v.as_str()).ok_or("Empty transactions array")?.to_string();

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
        let action = SolanaRequestHandler::parse_sign_message(Chain::Solana, params).unwrap();
        match action {
            WalletConnectAction::SignMessage { chain, sign_type, data } => {
                assert_eq!(chain, Chain::Solana);
                assert!(matches!(sign_type, SignDigestType::Base58));
                assert_eq!(data, "Hello");
            }
            _ => panic!("Expected SignMessage action"),
        }
    }

    #[test]
    fn test_parse_sign_transaction() {
        let params = serde_json::from_str(r#"{"transaction":"base64data"}"#).unwrap();
        let action = SolanaRequestHandler::parse_sign_transaction(Chain::Solana, params).unwrap();
        match action {
            WalletConnectAction::SignTransaction { chain, transaction_type, .. } => {
                assert_eq!(chain, Chain::Solana);
                assert!(matches!(
                    transaction_type,
                    WalletConnectTransactionType::Solana {
                        output_type: TransferDataOutputType::Signature
                    }
                ));
            }
            _ => panic!("Expected SignTransaction action"),
        }
    }
}
