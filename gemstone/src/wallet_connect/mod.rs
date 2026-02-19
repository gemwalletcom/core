use std::collections::HashMap;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use hex::FromHex;
use primitives::{Chain, WCEthereumTransaction, WalletConnectRequest, WalletConnectionVerificationStatus};

fn current_timestamp() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

use crate::{
    GemstoneError,
    message::sign_type::{SignDigestType, SignMessage},
    siwe::SiweMessage,
};

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

const TRON_METHOD_VERSION_KEY: &str = "tron_method_version";
const TRON_METHOD_VERSION_VALUE: &str = "v1";

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
    use crate::message::signer::MessageSigner;
    use gem_tron::TronChainSigner;
    use primitives::{
        Asset, ChainSigner, GasPriceType, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, TransferDataExtra, TransferDataOutputType,
        WalletConnectionSessionAppMetadata,
    };

    const TEST_PRIVATE_KEY: [u8; 32] = [1u8; 32];

    fn response_json(response: &WalletConnectResponseType) -> serde_json::Value {
        match response {
            WalletConnectResponseType::Object { json } => serde_json::from_str(json).unwrap(),
            WalletConnectResponseType::String { value } => serde_json::Value::String(value.clone()),
        }
    }

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
    fn parse_tron_sign_message_and_sign() {
        let wallet_connect = WalletConnect::new();
        let params = include_str!("./test/tron_sign_message.json");
        let raw_params = serde_json::to_string(&params.trim()).unwrap();
        let request = WalletConnectRequest::mock("tron_signMessage", &raw_params, Some("tron:0x2b6653dc"));

        let data = "This is a message to be signed for Tron".to_string();
        let action = WalletConnectRequestHandler::parse_request(request).unwrap();
        assert_eq!(
            action,
            WalletConnectAction::SignMessage {
                chain: Chain::Tron,
                sign_type: SignDigestType::TronPersonal,
                data: data.clone(),
            }
        );

        let sign_message = wallet_connect.decode_sign_message(Chain::Tron, SignDigestType::TronPersonal, data);
        let signer = MessageSigner::new(sign_message);
        let signature = signer.sign(TEST_PRIVATE_KEY.to_vec()).unwrap();

        assert_eq!(
            signature,
            "0xa0cbc20e8f0a9c19dd3d97e15fd99eee49edb8c0bcca52b684bbf13e1344b99670201d57633881cb20b0c00b626397530e3165049044b2fa4089840cf41a0a761b"
        );

        let response = wallet_connect.encode_sign_message(Chain::Tron, signature.clone());
        let expected: serde_json::Value = serde_json::from_str(include_str!("./test/tron_sign_message_response.json")).unwrap();
        assert_eq!(response_json(&response), expected);
    }

    #[test]
    fn parse_tron_sign_transaction_and_sign() {
        let wallet_connect = WalletConnect::new();
        let params = include_str!("./test/tron_sign_transaction.json");
        let expected_data: serde_json::Value = serde_json::from_str(params.trim()).unwrap();
        let expected_data = expected_data.to_string();
        let request = WalletConnectRequest::mock("tron_signTransaction", &serde_json::to_string(&params.trim()).unwrap(), Some("tron:0x2b6653dc"));

        let action = WalletConnectRequestHandler::parse_request(request).unwrap();
        assert_eq!(
            action,
            WalletConnectAction::SignTransaction {
                chain: Chain::Tron,
                transaction_type: WalletConnectTransactionType::Tron {
                    output_type: TransferDataOutputType::EncodedTransaction,
                },
                data: expected_data.clone(),
            }
        );

        let input = TransactionLoadInput {
            input_type: TransactionInputType::Generic(
                Asset::from_chain(Chain::Tron),
                WalletConnectionSessionAppMetadata::mock(),
                TransferDataExtra::mock_encoded_transaction(expected_data.as_bytes().to_vec()),
            ),
            sender_address: "TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM".to_string(),
            destination_address: "".to_string(),
            value: "0".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::mock_tron(),
        };

        let signature_payload = TronChainSigner.sign_data(&input, &TEST_PRIVATE_KEY).unwrap();
        let value: serde_json::Value = serde_json::from_str(&signature_payload).unwrap();

        assert_eq!(
            value["signature"][0].as_str().unwrap(),
            "943d286dfd1fb6a2cd31c9af7a6cfd23ee062ec2e0abcf82c7daa0c7bb43ab04458e0e88ebe3a94060122cccc8fb4395e5eb922720327df04ae840139c729a1f00"
        );
        assert!(value["txID"].as_str().is_some());
        assert!(value["raw_data_hex"].as_str().is_some());
        assert_eq!(value["visible"], serde_json::Value::Bool(false));

        let response = wallet_connect.encode_sign_transaction(Chain::Tron, signature_payload.clone());
        let expected: serde_json::Value = serde_json::from_str(include_str!("./test/tron_sign_transaction_response.json")).unwrap();
        assert_eq!(response_json(&response), expected);
    }

    #[test]
    fn parse_tron_sign_transaction_nested_and_sign() {
        let params = include_str!("./test/tron_sign_transaction_nested.json");
        let expected_data: serde_json::Value = serde_json::from_str(params.trim()).unwrap();
        let expected_data = expected_data.to_string();

        let input = TransactionLoadInput {
            input_type: TransactionInputType::Generic(
                Asset::from_chain(Chain::Tron),
                WalletConnectionSessionAppMetadata::mock(),
                TransferDataExtra::mock_encoded_transaction(expected_data.as_bytes().to_vec()),
            ),
            sender_address: "TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM".to_string(),
            destination_address: "".to_string(),
            value: "0".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::mock_tron(),
        };

        let signature_payload = TronChainSigner.sign_data(&input, &TEST_PRIVATE_KEY).unwrap();
        let value: serde_json::Value = serde_json::from_str(&signature_payload).unwrap();

        assert_eq!(
            value["signature"][0].as_str().unwrap(),
            "943d286dfd1fb6a2cd31c9af7a6cfd23ee062ec2e0abcf82c7daa0c7bb43ab04458e0e88ebe3a94060122cccc8fb4395e5eb922720327df04ae840139c729a1f00"
        );
        assert!(value["txID"].as_str().is_some());
        assert!(value["raw_data_hex"].as_str().is_some());
        assert_eq!(value["visible"], serde_json::Value::Bool(false));
    }

    #[test]
    fn parse_tron_send_transaction() {
        let params = include_str!("./test/tron_send_transaction.json");
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

    #[test]
    fn test_config_session_properties() {
        let wc = WalletConnect::new();
        let tron = vec![Chain::Tron.to_string()];

        let result = wc.config_session_properties(HashMap::new(), tron.clone());
        assert_eq!(result.get("tron_method_version").unwrap(), "v1");

        let mut props = HashMap::new();
        props.insert("tron_method_version".to_string(), "v2".to_string());
        let result = wc.config_session_properties(props, tron);
        assert_eq!(result.get("tron_method_version").unwrap(), "v2");

        let result = wc.config_session_properties(HashMap::new(), vec![Chain::Ethereum.to_string()]);
        assert!(result.get("tron_method_version").is_none());
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

    pub fn parse_request(&self, topic: String, method: String, params: String, chain_id: String, domain: String) -> Result<WalletConnectAction, GemstoneError> {
        let request = WalletConnectRequest {
            topic,
            method,
            params,
            chain_id: Some(chain_id),
            domain,
        };
        WalletConnectRequestHandler::parse_request(request).map_err(|e| GemstoneError::AnyError { msg: e })
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

    pub fn validate_sign_message(&self, chain: Chain, sign_type: SignDigestType, data: String) -> Result<(), GemstoneError> {
        match sign_type {
            SignDigestType::Eip712 => {
                let expected_chain_id = chain.network_id().parse::<u64>().map_err(|_| GemstoneError::AnyError {
                    msg: format!("Chain {} does not have a numeric network ID", chain),
                })?;
                gem_evm::eip712::validate_eip712_chain_id(&data, expected_chain_id).map_err(|e| GemstoneError::AnyError { msg: e })
            }
            SignDigestType::TonPersonal => {
                gem_ton::signer::TonSignMessageData::from_bytes(data.as_bytes())?;
                Ok(())
            }
            SignDigestType::Eip191
            | SignDigestType::Base58
            | SignDigestType::SuiPersonal
            | SignDigestType::Siwe
            | SignDigestType::BitcoinPersonal
            | SignDigestType::TronPersonal => Ok(()),
        }
    }

    pub fn validate_send_transaction(&self, transaction_type: WalletConnectTransactionType, data: String) -> Result<(), GemstoneError> {
        let WalletConnectTransactionType::Ton { .. } = transaction_type else {
            return Ok(());
        };

        let json: serde_json::Value = serde_json::from_str(&data).map_err(|_| GemstoneError::AnyError { msg: "Invalid JSON".to_string() })?;

        if let Some(valid_until) = json.get("valid_until").and_then(|v| v.as_i64())
            && current_timestamp() >= valid_until
        {
            return Err(GemstoneError::AnyError {
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

    pub fn config_session_properties(&self, properties: HashMap<String, String>, chains: Vec<String>) -> HashMap<String, String> {
        let mut result = properties;
        let chains: Vec<Chain> = chains.iter().filter_map(|c| Chain::from_str(c).ok()).collect();
        if chains.contains(&Chain::Tron) && !result.contains_key(TRON_METHOD_VERSION_KEY) {
            result.insert(TRON_METHOD_VERSION_KEY.to_string(), TRON_METHOD_VERSION_VALUE.to_string());
        }
        result
    }

    pub fn decode_send_transaction(&self, transaction_type: WalletConnectTransactionType, data: String) -> Result<WalletConnectTransaction, GemstoneError> {
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
                    .ok_or_else(|| GemstoneError::AnyError {
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
                    .ok_or_else(|| GemstoneError::AnyError {
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
                    .ok_or_else(|| GemstoneError::AnyError {
                        msg: "Missing messages field".to_string(),
                    })?
                    .to_string();

                Ok(WalletConnectTransaction::Ton { messages, output_type })
            }
            WalletConnectTransactionType::Bitcoin { output_type } => Ok(WalletConnectTransaction::Bitcoin { data, output_type }),
            WalletConnectTransactionType::Tron { output_type } => Ok(WalletConnectTransaction::Tron { data, output_type }),
        }
    }
}
