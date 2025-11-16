use crate::message::sign_type::{SignDigestType, SignMessage};
use hex::FromHex;
use primitives::{Chain, WCEthereumTransaction, WalletConnectRequest, WalletConnectionVerificationStatus};
use std::str::FromStr;

pub mod actions;
pub mod request_handler;
pub mod response_handler;
pub mod verifier;

pub use actions::{
    WCEthereumTransactionData, WCSolanaTransactionData, WCSuiTransactionData, WalletConnectAction, WalletConnectChainOperation, WalletConnectTransaction,
    WalletConnectTransactionType,
};
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

    pub fn parse_request(&self, topic: String, method: String, params: String, chain_id: String) -> Result<WalletConnectAction, crate::GemstoneError> {
        let request = WalletConnectRequest {
            topic,
            method,
            params,
            chain_id: Some(chain_id),
        };
        WalletConnectRequestHandler::parse_request(request).map_err(|e| crate::GemstoneError::AnyError { msg: e })
    }

    pub fn validate_origin(
        &self,
        metadata_url: String,
        origin: Option<String>,
        validation: WalletConnectionVerificationStatus,
    ) -> WalletConnectionVerificationStatus {
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

    pub fn decode_sign_message(&self, sign_type: SignDigestType, data: String) -> SignMessage {
        let message_data = if let Some(stripped) = data.strip_prefix("0x") {
            Vec::from_hex(stripped).unwrap_or_else(|_| data.as_bytes().to_vec())
        } else {
            Vec::from_hex(&data).unwrap_or_else(|_| data.as_bytes().to_vec())
        };

        SignMessage { sign_type, data: message_data }
    }

    pub fn decode_send_transaction(
        &self,
        transaction_type: WalletConnectTransactionType,
        data: String,
    ) -> Result<WalletConnectTransaction, crate::GemstoneError> {
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

                let wallet_address = json
                    .get("account")
                    .or_else(|| json.get("address"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                Ok(WalletConnectTransaction::Sui {
                    data: actions::WCSuiTransactionData { transaction, wallet_address },
                    output_type,
                })
            }
        }
    }
}

