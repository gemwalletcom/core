use crate::message::sign_type::SignDigestType;
use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::wallet_connect::handler_traits::ChainRequestHandler;
use primitives::{Chain, TransferDataOutputType, ValueAccess};
use serde_json::Value;

pub struct SolanaRequestHandler;

impl ChainRequestHandler for SolanaRequestHandler {
    fn parse_sign_message(_chain: Chain, params: Value, _domain: &str) -> Result<WalletConnectAction, String> {
        let message = params.get_value("message")?.string()?.to_string();

        Ok(WalletConnectAction::SignMessage {
            chain: Chain::Solana,
            sign_type: SignDigestType::Base58,
            data: message,
        })
    }

    fn parse_sign_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_value("transaction")?.string()?;

        Ok(WalletConnectAction::SignTransaction {
            chain: Chain::Solana,
            transaction_type: WalletConnectTransactionType::Solana {
                output_type: TransferDataOutputType::Signature,
            },
            data: params.to_string(),
        })
    }

    fn parse_send_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_value("transaction")?.string()?;

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

        let params: serde_json::Value = serde_json::from_str(params_json).unwrap();
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
