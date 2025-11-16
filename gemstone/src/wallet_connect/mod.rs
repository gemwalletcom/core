use crate::message::sign_type::{SignDigestType, SignMessage};
use hex::FromHex;
use primitives::{Chain, ChainType, WCEthereumTransaction, WalletConnectCAIP2, WalletConnectRequest, WalletConnectionVerificationStatus};
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

// CAIP-2 https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-2.md
pub fn get_namespace(chain: Chain) -> Option<String> {
    match chain.chain_type() {
        ChainType::Ethereum => Some(WalletConnectCAIP2::Eip155.as_ref().to_string()),
        ChainType::Solana => Some(WalletConnectCAIP2::Solana.as_ref().to_string()),
        ChainType::Cosmos => Some(format!("{}:{}", WalletConnectCAIP2::Cosmos.as_ref(), chain.network_id())),
        ChainType::Algorand => Some(WalletConnectCAIP2::Algorand.as_ref().to_string()),
        ChainType::Sui => Some(WalletConnectCAIP2::Sui.as_ref().to_string()),
        ChainType::Bitcoin
        | ChainType::Ton
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Xrp
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Polkadot
        | ChainType::Cardano
        | ChainType::HyperCore => None,
    }
}

pub fn get_chain_type(namespace: String) -> Option<ChainType> {
    match WalletConnectCAIP2::from_str(&namespace).ok()? {
        WalletConnectCAIP2::Eip155 => Some(ChainType::Ethereum),
        WalletConnectCAIP2::Solana => Some(ChainType::Solana),
        WalletConnectCAIP2::Cosmos => Some(ChainType::Cosmos),
        WalletConnectCAIP2::Algorand => Some(ChainType::Algorand),
        WalletConnectCAIP2::Sui => Some(ChainType::Sui),
    }
}

pub fn get_chain(namespace: String, reference: String) -> Option<Chain> {
    let namespace = WalletConnectCAIP2::from_str(&namespace).ok()?;
    match namespace {
        WalletConnectCAIP2::Eip155 | WalletConnectCAIP2::Cosmos => {
            let chain_type = get_chain_type(namespace.as_ref().to_string())?;
            Chain::all()
                .into_iter()
                .filter(|chain| chain.chain_type() == chain_type && chain.network_id() == reference)
                .collect::<Vec<_>>()
                .first()
                .cloned()
        }
        WalletConnectCAIP2::Solana => Some(Chain::Solana),
        WalletConnectCAIP2::Algorand => Some(Chain::Algorand),
        WalletConnectCAIP2::Sui => Some(Chain::Sui),
    }
}

// CAIP-20 https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-20.md
pub fn get_reference(chain: Chain) -> Option<String> {
    match chain.chain_type() {
        ChainType::Ethereum => Some(chain.network_id().to_string()),
        ChainType::Solana => Some(chain.network_id().chars().take(32).collect()),
        ChainType::Cosmos => get_namespace(chain).map(|namespace| format!("{}:{}", namespace, chain.network_id())),
        ChainType::Algorand => Some("wGHE2Pwdvd7S12BL5FaOP20EGYesN73k".to_string()),
        ChainType::Sui => Some("mainnet".to_string()),
        ChainType::Bitcoin
        | ChainType::Ton
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Xrp
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Polkadot
        | ChainType::Cardano
        | ChainType::HyperCore => None,
    }
}

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
        get_namespace(chain)
    }

    pub fn get_reference(&self, chain: String) -> Option<String> {
        let chain = Chain::from_str(&chain).ok()?;
        get_reference(chain)
    }

    pub fn get_chain(&self, caip2: String, caip10: String) -> Option<String> {
        Some(get_chain(caip2, caip10)?.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, ChainType};

    #[test]
    fn test_get_chain_type_basic() {
        assert_eq!(get_chain_type("eip155".to_string()), Some(ChainType::Ethereum));
        assert_eq!(get_chain_type("solana".to_string()), Some(ChainType::Solana));
        assert_eq!(get_chain_type("cosmos".to_string()), Some(ChainType::Cosmos));
        assert_eq!(get_chain_type("algorand".to_string()), Some(ChainType::Algorand));
        assert_eq!(get_chain_type("unknown".to_string()), None);
    }

    #[test]
    fn test_get_chain_eip155() {
        assert_eq!(get_chain("eip155".to_string(), "1".to_string()), Some(Chain::Ethereum));
        assert_eq!(get_chain("eip155".to_string(), "56".to_string()), Some(Chain::SmartChain));
    }

    #[test]
    fn test_get_chain_solana() {
        let chain = get_chain("solana".to_string(), "ignored".to_string());
        assert_eq!(chain, Some(Chain::Solana));
    }
}
