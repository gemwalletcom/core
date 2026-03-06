use std::collections::HashMap;
use std::str::FromStr;

use gem_wallet_connect::{
    SignMessageValidation, WalletConnectRequestHandler, WalletConnectResponseHandler, WalletConnectVerifier, config_session_properties, decode_sign_message,
    validate_send_transaction, validate_sign_message,
};
use primitives::{Chain, TransferDataOutputType, WCEthereumTransaction, WalletConnectRequest, WalletConnectionVerificationStatus};

use crate::{
    GemstoneError,
    message::sign_type::{SignDigestType, SignMessage},
};

// UniFFI remote enum declaration
#[uniffi::remote(Enum)]
pub enum WalletConnectionVerificationStatus {
    Verified,
    Unknown,
    Invalid,
    Malicious,
}

// UniFFI types

#[derive(Debug, Clone, uniffi::Record)]
pub struct WCEthereumTransactionData {
    pub chain_id: Option<String>,
    pub from: String,
    pub to: String,
    pub value: Option<String>,
    pub gas: Option<String>,
    pub gas_limit: Option<String>,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub nonce: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct WCSolanaTransactionData {
    pub transaction: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct WCSuiTransactionData {
    pub transaction: String,
    pub wallet_address: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum WalletConnectAction {
    SignMessage {
        chain: Chain,
        sign_type: SignDigestType,
        data: String,
    },
    SignTransaction {
        chain: Chain,
        transaction_type: WalletConnectTransactionType,
        data: String,
    },
    SendTransaction {
        chain: Chain,
        transaction_type: WalletConnectTransactionType,
        data: String,
    },
    ChainOperation {
        operation: WalletConnectChainOperation,
    },
    Unsupported {
        method: String,
    },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum WalletConnectTransactionType {
    Ethereum,
    Solana { output_type: TransferDataOutputType },
    Sui { output_type: TransferDataOutputType },
    Ton { output_type: TransferDataOutputType },
    Bitcoin { output_type: TransferDataOutputType },
    Tron { output_type: TransferDataOutputType },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum WalletConnectChainOperation {
    AddChain,
    SwitchChain,
    GetChainId,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum WalletConnectTransaction {
    Ethereum {
        data: WCEthereumTransactionData,
    },
    Solana {
        data: WCSolanaTransactionData,
        output_type: TransferDataOutputType,
    },
    Sui {
        data: WCSuiTransactionData,
        output_type: TransferDataOutputType,
    },
    Ton {
        messages: String,
        output_type: TransferDataOutputType,
    },
    Bitcoin {
        data: String,
        output_type: TransferDataOutputType,
    },
    Tron {
        data: String,
        output_type: TransferDataOutputType,
    },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum WalletConnectResponseType {
    String { value: String },
    Object { json: String },
}

// From conversions: primitives -> UniFFI

impl From<WCEthereumTransaction> for WCEthereumTransactionData {
    fn from(tx: WCEthereumTransaction) -> Self {
        Self {
            chain_id: tx.chain_id,
            from: tx.from,
            to: tx.to,
            value: tx.value,
            gas: tx.gas,
            gas_limit: tx.gas_limit,
            gas_price: tx.gas_price,
            max_fee_per_gas: tx.max_fee_per_gas,
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
            nonce: tx.nonce,
            data: tx.data,
        }
    }
}

// From conversions: gem_wallet_connect -> UniFFI

impl From<gem_wallet_connect::SignDigestType> for SignDigestType {
    fn from(t: gem_wallet_connect::SignDigestType) -> Self {
        match t {
            gem_wallet_connect::SignDigestType::Eip191 => Self::Eip191,
            gem_wallet_connect::SignDigestType::Eip712 => Self::Eip712,
            gem_wallet_connect::SignDigestType::Base58 => Self::Base58,
            gem_wallet_connect::SignDigestType::SuiPersonal => Self::SuiPersonal,
            gem_wallet_connect::SignDigestType::Siwe => Self::Siwe,
            gem_wallet_connect::SignDigestType::TonPersonal => Self::TonPersonal,
            gem_wallet_connect::SignDigestType::BitcoinPersonal => Self::BitcoinPersonal,
            gem_wallet_connect::SignDigestType::TronPersonal => Self::TronPersonal,
        }
    }
}

impl From<SignDigestType> for gem_wallet_connect::SignDigestType {
    fn from(t: SignDigestType) -> Self {
        match t {
            SignDigestType::Eip191 => Self::Eip191,
            SignDigestType::Eip712 => Self::Eip712,
            SignDigestType::Base58 => Self::Base58,
            SignDigestType::SuiPersonal => Self::SuiPersonal,
            SignDigestType::Siwe => Self::Siwe,
            SignDigestType::TonPersonal => Self::TonPersonal,
            SignDigestType::BitcoinPersonal => Self::BitcoinPersonal,
            SignDigestType::TronPersonal => Self::TronPersonal,
        }
    }
}

impl From<gem_wallet_connect::WalletConnectTransactionType> for WalletConnectTransactionType {
    fn from(t: gem_wallet_connect::WalletConnectTransactionType) -> Self {
        match t {
            gem_wallet_connect::WalletConnectTransactionType::Ethereum => Self::Ethereum,
            gem_wallet_connect::WalletConnectTransactionType::Solana { output_type } => Self::Solana { output_type },
            gem_wallet_connect::WalletConnectTransactionType::Sui { output_type } => Self::Sui { output_type },
            gem_wallet_connect::WalletConnectTransactionType::Ton { output_type } => Self::Ton { output_type },
            gem_wallet_connect::WalletConnectTransactionType::Bitcoin { output_type } => Self::Bitcoin { output_type },
            gem_wallet_connect::WalletConnectTransactionType::Tron { output_type } => Self::Tron { output_type },
        }
    }
}

impl From<WalletConnectTransactionType> for gem_wallet_connect::WalletConnectTransactionType {
    fn from(t: WalletConnectTransactionType) -> Self {
        match t {
            WalletConnectTransactionType::Ethereum => Self::Ethereum,
            WalletConnectTransactionType::Solana { output_type } => Self::Solana { output_type },
            WalletConnectTransactionType::Sui { output_type } => Self::Sui { output_type },
            WalletConnectTransactionType::Ton { output_type } => Self::Ton { output_type },
            WalletConnectTransactionType::Bitcoin { output_type } => Self::Bitcoin { output_type },
            WalletConnectTransactionType::Tron { output_type } => Self::Tron { output_type },
        }
    }
}

impl From<gem_wallet_connect::WalletConnectChainOperation> for WalletConnectChainOperation {
    fn from(op: gem_wallet_connect::WalletConnectChainOperation) -> Self {
        match op {
            gem_wallet_connect::WalletConnectChainOperation::AddChain => Self::AddChain,
            gem_wallet_connect::WalletConnectChainOperation::SwitchChain => Self::SwitchChain,
            gem_wallet_connect::WalletConnectChainOperation::GetChainId => Self::GetChainId,
        }
    }
}

impl From<gem_wallet_connect::WalletConnectAction> for WalletConnectAction {
    fn from(action: gem_wallet_connect::WalletConnectAction) -> Self {
        match action {
            gem_wallet_connect::WalletConnectAction::SignMessage { chain, sign_type, data } => Self::SignMessage {
                chain,
                sign_type: sign_type.into(),
                data,
            },
            gem_wallet_connect::WalletConnectAction::SignTransaction { chain, transaction_type, data } => Self::SignTransaction {
                chain,
                transaction_type: transaction_type.into(),
                data,
            },
            gem_wallet_connect::WalletConnectAction::SendTransaction { chain, transaction_type, data } => Self::SendTransaction {
                chain,
                transaction_type: transaction_type.into(),
                data,
            },
            gem_wallet_connect::WalletConnectAction::ChainOperation { operation } => Self::ChainOperation { operation: operation.into() },
            gem_wallet_connect::WalletConnectAction::Unsupported { method } => Self::Unsupported { method },
        }
    }
}

impl From<gem_wallet_connect::WCEthereumTransactionData> for WCEthereumTransactionData {
    fn from(d: gem_wallet_connect::WCEthereumTransactionData) -> Self {
        Self {
            chain_id: d.chain_id,
            from: d.from,
            to: d.to,
            value: d.value,
            gas: d.gas,
            gas_limit: d.gas_limit,
            gas_price: d.gas_price,
            max_fee_per_gas: d.max_fee_per_gas,
            max_priority_fee_per_gas: d.max_priority_fee_per_gas,
            nonce: d.nonce,
            data: d.data,
        }
    }
}

impl From<gem_wallet_connect::WalletConnectTransaction> for WalletConnectTransaction {
    fn from(t: gem_wallet_connect::WalletConnectTransaction) -> Self {
        match t {
            gem_wallet_connect::WalletConnectTransaction::Ethereum { data } => Self::Ethereum { data: data.into() },
            gem_wallet_connect::WalletConnectTransaction::Solana { data, output_type } => Self::Solana {
                data: WCSolanaTransactionData { transaction: data.transaction },
                output_type,
            },
            gem_wallet_connect::WalletConnectTransaction::Sui { data, output_type } => Self::Sui {
                data: WCSuiTransactionData {
                    transaction: data.transaction,
                    wallet_address: data.wallet_address,
                },
                output_type,
            },
            gem_wallet_connect::WalletConnectTransaction::Ton { messages, output_type } => Self::Ton { messages, output_type },
            gem_wallet_connect::WalletConnectTransaction::Bitcoin { data, output_type } => Self::Bitcoin { data, output_type },
            gem_wallet_connect::WalletConnectTransaction::Tron { data, output_type } => Self::Tron { data, output_type },
        }
    }
}

impl From<gem_wallet_connect::WalletConnectResponseType> for WalletConnectResponseType {
    fn from(r: gem_wallet_connect::WalletConnectResponseType) -> Self {
        match r {
            gem_wallet_connect::WalletConnectResponseType::String { value } => Self::String { value },
            gem_wallet_connect::WalletConnectResponseType::Object { json } => Self::Object { json },
        }
    }
}

// WalletConnect UniFFI object

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

    pub fn parse_request(&self, topic: String, method: String, params: String, chain_id: String, domain: String) -> Result<WalletConnectAction, GemstoneError> {
        let request = WalletConnectRequest {
            topic,
            method,
            params,
            chain_id: Some(chain_id),
            domain,
        };
        let action = WalletConnectRequestHandler::parse_request(request).map_err(|e| GemstoneError::AnyError { msg: e })?;
        Ok(action.into())
    }

    pub fn validate_origin(&self, metadata_url: String, origin: Option<String>, validation: WalletConnectionVerificationStatus) -> WalletConnectionVerificationStatus {
        WalletConnectVerifier::validate_origin(metadata_url, origin, validation)
    }

    pub fn encode_sign_message(&self, chain: Chain, signature: String) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_sign_message(chain.chain_type(), signature).into()
    }

    pub fn encode_sign_transaction(&self, chain: Chain, transaction_id: String) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_sign_transaction(chain.chain_type(), transaction_id).into()
    }

    pub fn encode_send_transaction(&self, chain: Chain, transaction_id: String) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_send_transaction(chain.chain_type(), transaction_id).into()
    }

    pub fn validate_sign_message(&self, chain: Chain, sign_type: SignDigestType, data: String, session_domain: String) -> Result<(), GemstoneError> {
        let crate_sign_type: gem_wallet_connect::SignDigestType = sign_type.into();
        let input = SignMessageValidation {
            chain,
            sign_type: &crate_sign_type,
            data: &data,
            session_domain: &session_domain,
        };
        validate_sign_message(&input).map_err(|e| GemstoneError::AnyError { msg: e })
    }

    pub fn validate_send_transaction(&self, transaction_type: WalletConnectTransactionType, data: String) -> Result<(), GemstoneError> {
        let crate_type: gem_wallet_connect::WalletConnectTransactionType = transaction_type.into();
        validate_send_transaction(&crate_type, &data).map_err(|e| GemstoneError::AnyError { msg: e })
    }

    pub fn decode_sign_message(&self, chain: Chain, sign_type: SignDigestType, data: String) -> SignMessage {
        let crate_sign_type: gem_wallet_connect::SignDigestType = sign_type.into();
        let result = decode_sign_message(chain, crate_sign_type, data);
        SignMessage {
            chain: result.chain,
            sign_type: result.sign_type.into(),
            data: result.data,
        }
    }

    pub fn config_session_properties(&self, properties: HashMap<String, String>, chains: Vec<String>) -> HashMap<String, String> {
        let chains = gem_wallet_connect::session::parse_chains(&chains);
        config_session_properties(properties, &chains)
    }

    pub fn decode_send_transaction(&self, transaction_type: WalletConnectTransactionType, data: String) -> Result<WalletConnectTransaction, GemstoneError> {
        let crate_type: gem_wallet_connect::WalletConnectTransactionType = transaction_type.into();
        let crate_result = WalletConnectRequestHandler::decode_send_transaction(crate_type, data).map_err(|e| GemstoneError::AnyError { msg: e })?;
        Ok(crate_result.into())
    }
}
