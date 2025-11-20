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
        params.get("transaction").and_then(|v| v.as_str()).ok_or("Missing transaction parameter")?;

        Ok(WalletConnectAction::SignTransaction {
            chain: Chain::Solana,
            transaction_type: WalletConnectTransactionType::Solana {
                output_type: TransferDataOutputType::Signature,
            },
            data: params.to_string(),
        })
    }

    fn parse_send_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get("transaction").and_then(|v| v.as_str()).ok_or("Missing transaction parameter")?;

        Ok(WalletConnectAction::SendTransaction {
            chain: Chain::Solana,
            transaction_type: WalletConnectTransactionType::Solana {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data: params.to_string(),
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
    fn test_sign_transaction() {
        let params_json = r#"{
            "feePayer":"7GgpgQLnhfBw9ukjQuTNhbrEJMmPNV5c9iaPUpXm1dKM",
            "instructions":[{
                "data":"3Bxs412MvVNQj175",
                "keys":[
                    {"isSigner":true,"isWritable":true,"pubkey":"7GgpgQLnhfBw9ukjQuTNhbrEJMmPNV5c9iaPUpXm1dKM"},
                    {"isSigner":false,"isWritable":true,"pubkey":"4mCUu39wVr8jQ1rCNpPzru86EVvN24a3bgi6RSSTQqha"}
                ],
                "programId":"11111111111111111111111111111111"
            }],
            "partialSignatures":[],
            "recentBlockhash":"7RB8by3EMX4UqUSWx2wWwRGeNPaBjWSkpFxEdTuVWA8s",
            "transaction":"AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDXSrcUs6yh6vSKrAi1IfpC6eB2ZwoCahNCXdBfn76Kfg35ZkUm4Dl6OeRysXE26nNgky3Ei9W8GElwuKbUNB9CQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAX1eEJ5Ueb/dix/IMPlhpv/ubGMU7MhDs5fLp4r4KwFQBAgIAAQwCAAAAAQAAAAAAAAA="
        }"#;

        let params = serde_json::from_str(params_json).unwrap();
        let action = SolanaRequestHandler::parse_sign_transaction(Chain::Solana, params).unwrap();

        match action {
            WalletConnectAction::SignTransaction { chain, transaction_type, data } => {
                assert_eq!(chain, Chain::Solana);
                assert!(matches!(
                    transaction_type,
                    WalletConnectTransactionType::Solana {
                        output_type: TransferDataOutputType::Signature
                    }
                ));

                let parsed_data: serde_json::Value = serde_json::from_str(&data).expect("Data should be valid JSON");
                assert_eq!(
                    parsed_data.get("transaction").and_then(|v| v.as_str()),
                    Some(
                        "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDXSrcUs6yh6vSKrAi1IfpC6eB2ZwoCahNCXdBfn76Kfg35ZkUm4Dl6OeRysXE26nNgky3Ei9W8GElwuKbUNB9CQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAX1eEJ5Ueb/dix/IMPlhpv/ubGMU7MhDs5fLp4r4KwFQBAgIAAQwCAAAAAQAAAAAAAAA="
                    )
                );
                assert_eq!(
                    parsed_data.get("feePayer").and_then(|v| v.as_str()),
                    Some("7GgpgQLnhfBw9ukjQuTNhbrEJMmPNV5c9iaPUpXm1dKM")
                );
            }
            _ => panic!("Expected SignTransaction action"),
        }
    }
}
