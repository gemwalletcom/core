use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use hex::FromHex;
use primitives::{Chain, WCEthereumTransaction, WalletConnectRequest, WalletConnectionVerificationStatus};

fn current_timestamp() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

use crate::message::sign_type::{SignDigestType, SignMessage};
use crate::siwe::SiweMessage;

pub mod actions;
pub mod handler_traits;
pub mod request_handler;
pub mod response_handler;
pub mod verifier;

pub use actions::{
    WCEthereumTransactionData, WCSolanaTransactionData, WCSuiTransactionData, WalletConnectAction, WalletConnectChainOperation, WalletConnectTransaction,
    WalletConnectTransactionType,
};
pub use handler_traits::{ChainRequestHandler, ChainResponseHandler};
pub use request_handler::WalletConnectRequestHandler;
pub use response_handler::*;
pub use verifier::*;

#[derive(uniffi::Object)]
pub struct WalletConnect {}

impl Default for WalletConnect {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_siwe_message() -> String {
        [
            "login.xyz wants you to sign in with your Ethereum account:",
            "0x6dD7802E6d44bE89a789C4bD60bD511B68F41c7c",
            "",
            "Sign in with Ethereum to the app.",
            "",
            "URI: https://login.xyz",
            "Version: 1",
            "Chain ID: 1",
            "Nonce: 8hK9pX32",
            "Issued At: 2024-04-01T12:00:00Z",
        ]
        .join("\n")
    }

    #[test]
    fn decode_sign_message_detects_siwe() {
        let wallet_connect = WalletConnect::new();
        let message = sample_siwe_message();

        let decoded = wallet_connect.decode_sign_message(Chain::Ethereum, SignDigestType::Eip191, message.clone());

        assert_eq!(decoded.sign_type, SignDigestType::Siwe);
        assert_eq!(decoded.data, message.into_bytes());
    }

    #[test]
    fn decode_sign_message_preserves_non_siwe() {
        let wallet_connect = WalletConnect::new();
        let message = "Hello world".to_string();

        let decoded = wallet_connect.decode_sign_message(Chain::Ethereum, SignDigestType::Eip191, message.clone());

        assert_eq!(decoded.sign_type, SignDigestType::Eip191);
        assert_eq!(decoded.data, message.into_bytes());
    }

    #[test]
    fn decode_sign_message_siwe_chain_mismatch() {
        let wallet_connect = WalletConnect::new();
        let message = sample_siwe_message();

        let decoded = wallet_connect.decode_sign_message(Chain::Polygon, SignDigestType::Eip191, message);

        assert_eq!(decoded.sign_type, SignDigestType::Eip191);
    }

    #[test]
    fn validate_ton_sign_message() {
        use gem_ton::signer::{TonSignDataPayload, TonSignMessageData};

        let wallet_connect = WalletConnect::new();

        // Missing type field in payload
        let ton_data = r#"{"payload":{"text":"Hello"},"domain":"example.com"}"#.to_string();
        assert!(wallet_connect.validate_sign_message(Chain::Ton, SignDigestType::TonPersonal, ton_data).is_err());

        // Unknown type
        let ton_data = r#"{"payload":{"type":"unknown"},"domain":"example.com"}"#.to_string();
        assert!(wallet_connect.validate_sign_message(Chain::Ton, SignDigestType::TonPersonal, ton_data).is_err());

        // Valid text type
        let ton_data = TonSignMessageData::new(TonSignDataPayload::Text { text: "Hello".to_string() }, "example.com".to_string());
        assert!(
            wallet_connect
                .validate_sign_message(Chain::Ton, SignDigestType::TonPersonal, String::from_utf8(ton_data.to_bytes()).unwrap())
                .is_ok()
        );

        // Valid binary type
        let ton_data = TonSignMessageData::new(TonSignDataPayload::Binary { bytes: "SGVsbG8=".to_string() }, "example.com".to_string());
        assert!(
            wallet_connect
                .validate_sign_message(Chain::Ton, SignDigestType::TonPersonal, String::from_utf8(ton_data.to_bytes()).unwrap())
                .is_ok()
        );

        // Valid cell type
        let ton_data = TonSignMessageData::new(TonSignDataPayload::Cell { cell: "te6c".to_string() }, "example.com".to_string());
        assert!(
            wallet_connect
                .validate_sign_message(Chain::Ton, SignDigestType::TonPersonal, String::from_utf8(ton_data.to_bytes()).unwrap())
                .is_ok()
        );
    }

    #[test]
    fn validate_ton_send_transaction() {
        let wallet_connect = WalletConnect::new();
        let ton_type = WalletConnectTransactionType::Ton {
            output_type: primitives::TransferDataOutputType::EncodedTransaction,
        };

        assert!(
            wallet_connect
                .validate_send_transaction(ton_type.clone(), r#"{"valid_until": 1234567890, "messages": []}"#.to_string())
                .is_err()
        );
        assert!(
            wallet_connect
                .validate_send_transaction(ton_type.clone(), r#"{"valid_until": 9999999999, "messages": []}"#.to_string())
                .is_ok()
        );
        assert!(wallet_connect.validate_send_transaction(ton_type, r#"{"messages": []}"#.to_string()).is_ok());
    }
}

#[uniffi::export]
impl WalletConnect {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_namespace(&self, chain: String) -> Option<String> {
        let chain = Chain::from_str(&chain).ok()?;
        primitives::WalletConnectCAIP2::get_namespace(chain)
    }

    pub fn get_reference(&self, chain: String) -> Option<String> {
        let chain = Chain::from_str(&chain).ok()?;
        primitives::WalletConnectCAIP2::get_reference(chain)
    }

    pub fn get_chain(&self, caip2: String, caip10: String) -> Option<String> {
        Some(primitives::WalletConnectCAIP2::get_chain(caip2, caip10)?.to_string())
    }

    pub fn parse_request(&self, topic: String, method: String, params: String, chain_id: String, domain: String) -> Result<WalletConnectAction, crate::GemstoneError> {
        let request = WalletConnectRequest {
            topic,
            method,
            params,
            chain_id: Some(chain_id),
            domain,
        };
        WalletConnectRequestHandler::parse_request(request).map_err(|e| crate::GemstoneError::AnyError { msg: e })
    }

    pub fn validate_origin(&self, metadata_url: String, origin: Option<String>, validation: WalletConnectionVerificationStatus) -> WalletConnectionVerificationStatus {
        WalletConnectVerifier::validate_origin(metadata_url, origin, validation)
    }

    pub fn encode_sign_message(&self, chain: Chain, signature: String) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_sign_message(chain.chain_type(), signature)
    }

    pub fn encode_sign_transaction(&self, chain: Chain, transaction_id: String) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_sign_transaction(chain.chain_type(), transaction_id)
    }

    pub fn encode_send_transaction(&self, chain: Chain, transaction_id: String) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_send_transaction(chain.chain_type(), transaction_id)
    }

    pub fn validate_sign_message(&self, chain: Chain, sign_type: SignDigestType, data: String) -> Result<(), crate::GemstoneError> {
        match sign_type {
            SignDigestType::Eip712 => {
                let expected_chain_id = chain.network_id().parse::<u64>().map_err(|_| crate::GemstoneError::AnyError {
                    msg: format!("Chain {} does not have a numeric network ID", chain),
                })?;
                gem_evm::eip712::validate_eip712_chain_id(&data, expected_chain_id).map_err(|e| crate::GemstoneError::AnyError { msg: e })
            }
            SignDigestType::TonPersonal => {
                gem_ton::signer::TonSignMessageData::from_bytes(data.as_bytes())?;
                Ok(())
            }
            SignDigestType::Eip191 | SignDigestType::Base58 | SignDigestType::SuiPersonal | SignDigestType::Siwe | SignDigestType::BitcoinPersonal => Ok(()),
        }
    }

    pub fn validate_send_transaction(&self, transaction_type: WalletConnectTransactionType, data: String) -> Result<(), crate::GemstoneError> {
        let WalletConnectTransactionType::Ton { .. } = transaction_type else {
            return Ok(());
        };

        let json: serde_json::Value = serde_json::from_str(&data).map_err(|_| crate::GemstoneError::AnyError { msg: "Invalid JSON".to_string() })?;

        if let Some(valid_until) = json.get("valid_until").and_then(|v| v.as_i64())
            && current_timestamp() >= valid_until
        {
            return Err(crate::GemstoneError::AnyError {
                msg: "Transaction expired".to_string(),
            });
        }

        Ok(())
    }

    pub fn decode_sign_message(&self, chain: Chain, sign_type: SignDigestType, data: String) -> SignMessage {
        let mut utf8_value = None;
        let message_data = if let Some(stripped) = data.strip_prefix("0x") {
            Vec::from_hex(stripped).unwrap_or_else(|_| data.as_bytes().to_vec())
        } else {
            utf8_value = Some(data.clone());
            data.into_bytes()
        };

        let raw_text = utf8_value.or_else(|| String::from_utf8(message_data.clone()).ok()).unwrap_or_default();

        if sign_type == SignDigestType::Eip191
            && let Some(siwe_message) = self.decode_siwe_message(chain, &raw_text, &message_data)
        {
            return siwe_message;
        }

        SignMessage {
            chain,
            sign_type,
            data: message_data,
        }
    }

    fn decode_siwe_message(&self, chain: Chain, raw_text: &str, message_data: &[u8]) -> Option<SignMessage> {
        let message = SiweMessage::try_parse(raw_text)?;
        message.validate(chain).ok()?;

        Some(SignMessage {
            chain,
            sign_type: SignDigestType::Siwe,
            data: message_data.to_vec(),
        })
    }

    pub fn decode_send_transaction(&self, transaction_type: WalletConnectTransactionType, data: String) -> Result<WalletConnectTransaction, crate::GemstoneError> {
        match transaction_type {
            WalletConnectTransactionType::Ethereum => {
                let tx: WCEthereumTransaction = serde_json::from_str(&data)?;
                Ok(WalletConnectTransaction::Ethereum { data: tx.into() })
            }
            WalletConnectTransactionType::Solana { output_type } => {
                let json: serde_json::Value = serde_json::from_str(&data)?;

                let transaction = json
                    .get("transaction")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::GemstoneError::AnyError {
                        msg: "Missing transaction field".to_string(),
                    })?
                    .to_string();

                Ok(WalletConnectTransaction::Solana {
                    data: actions::WCSolanaTransactionData { transaction },
                    output_type,
                })
            }
            WalletConnectTransactionType::Sui { output_type } => {
                let json: serde_json::Value = serde_json::from_str(&data)?;

                let transaction = json
                    .get("transaction")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::GemstoneError::AnyError {
                        msg: "Missing transaction field".to_string(),
                    })?
                    .to_string();

                let wallet_address = json.get("account").or_else(|| json.get("address")).and_then(|v| v.as_str()).unwrap_or_default().to_string();

                Ok(WalletConnectTransaction::Sui {
                    data: actions::WCSuiTransactionData { transaction, wallet_address },
                    output_type,
                })
            }
            WalletConnectTransactionType::Ton { output_type } => {
                let json: serde_json::Value = serde_json::from_str(&data)?;

                let messages = json
                    .get("messages")
                    .ok_or_else(|| crate::GemstoneError::AnyError {
                        msg: "Missing messages field".to_string(),
                    })?
                    .to_string();

                Ok(WalletConnectTransaction::Ton { messages, output_type })
            }
            WalletConnectTransactionType::Bitcoin { output_type } => Ok(WalletConnectTransaction::Bitcoin { data, output_type }),
        }
    }
}
