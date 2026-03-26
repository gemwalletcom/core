use gem_wallet_connect::session::parse_chains;
use gem_wallet_connect::{
    SignDigestType as WcSignDigestType, WCEthereumTransactionData as WcEthereumTransactionData, WalletConnectAction as WcWalletConnectAction,
    WalletConnectChainOperation as WcWalletConnectChainOperation, WalletConnectRequestHandler, WalletConnectResponseHandler,
    WalletConnectResponseType as WcWalletConnectResponseType, WalletConnectTransaction as WcWalletConnectTransaction,
    WalletConnectTransactionType as WcWalletConnectTransactionType, WalletConnectVerifier, config_session_properties,
};
use primitives::{Chain, SimulationResult, TransferDataOutputType, WCEthereumTransaction, WalletConnectRequest, WalletConnectionVerificationStatus};
use std::collections::HashMap;
use std::str::FromStr;

use crate::{
    GemstoneError,
    message::sign_type::{SignDigestType, SignMessage},
};

mod simulation;
mod simulation_client;
pub use simulation_client::WalletConnectSimulationClient;

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
    SignAllTransactions {
        chain: Chain,
        transaction_type: WalletConnectTransactionType,
        transactions: Vec<String>,
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
    SwitchChain { chain: Chain },
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

impl From<WcSignDigestType> for SignDigestType {
    fn from(t: WcSignDigestType) -> Self {
        match t {
            WcSignDigestType::Eip191 => Self::Eip191,
            WcSignDigestType::Eip712 => Self::Eip712,
            WcSignDigestType::Base58 => Self::Base58,
            WcSignDigestType::SuiPersonal => Self::SuiPersonal,
            WcSignDigestType::Siwe => Self::Siwe,
            WcSignDigestType::TonPersonal => Self::TonPersonal,
            WcSignDigestType::BitcoinPersonal => Self::BitcoinPersonal,
            WcSignDigestType::TronPersonal => Self::TronPersonal,
        }
    }
}

impl From<SignDigestType> for WcSignDigestType {
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

impl From<WcWalletConnectTransactionType> for WalletConnectTransactionType {
    fn from(t: WcWalletConnectTransactionType) -> Self {
        match t {
            WcWalletConnectTransactionType::Ethereum => Self::Ethereum,
            WcWalletConnectTransactionType::Solana { output_type } => Self::Solana { output_type },
            WcWalletConnectTransactionType::Sui { output_type } => Self::Sui { output_type },
            WcWalletConnectTransactionType::Ton { output_type } => Self::Ton { output_type },
            WcWalletConnectTransactionType::Bitcoin { output_type } => Self::Bitcoin { output_type },
            WcWalletConnectTransactionType::Tron { output_type } => Self::Tron { output_type },
        }
    }
}

impl From<WalletConnectTransactionType> for WcWalletConnectTransactionType {
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

impl From<WcWalletConnectChainOperation> for WalletConnectChainOperation {
    fn from(op: WcWalletConnectChainOperation) -> Self {
        match op {
            WcWalletConnectChainOperation::AddChain => Self::AddChain,
            WcWalletConnectChainOperation::SwitchChain { chain } => Self::SwitchChain { chain },
            WcWalletConnectChainOperation::GetChainId => Self::GetChainId,
        }
    }
}

impl From<WcWalletConnectAction> for WalletConnectAction {
    fn from(action: WcWalletConnectAction) -> Self {
        match action {
            WcWalletConnectAction::SignMessage { chain, sign_type, data } => Self::SignMessage {
                chain,
                sign_type: sign_type.into(),
                data,
            },
            WcWalletConnectAction::SignTransaction { chain, transaction_type, data } => Self::SignTransaction {
                chain,
                transaction_type: transaction_type.into(),
                data,
            },
            WcWalletConnectAction::SignAllTransactions { chain, transaction_type, transactions } => Self::SignAllTransactions {
                chain,
                transaction_type: transaction_type.into(),
                transactions,
            },
            WcWalletConnectAction::SendTransaction { chain, transaction_type, data } => Self::SendTransaction {
                chain,
                transaction_type: transaction_type.into(),
                data,
            },
            WcWalletConnectAction::ChainOperation { operation } => Self::ChainOperation { operation: operation.into() },
            WcWalletConnectAction::Unsupported { method } => Self::Unsupported { method },
        }
    }
}

impl From<WcEthereumTransactionData> for WCEthereumTransactionData {
    fn from(d: WcEthereumTransactionData) -> Self {
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

impl From<WcWalletConnectTransaction> for WalletConnectTransaction {
    fn from(t: WcWalletConnectTransaction) -> Self {
        match t {
            WcWalletConnectTransaction::Ethereum { data } => Self::Ethereum { data: data.into() },
            WcWalletConnectTransaction::Solana { data, output_type } => Self::Solana {
                data: WCSolanaTransactionData { transaction: data.transaction },
                output_type,
            },
            WcWalletConnectTransaction::Sui { data, output_type } => Self::Sui {
                data: WCSuiTransactionData {
                    transaction: data.transaction,
                    wallet_address: data.wallet_address,
                },
                output_type,
            },
            WcWalletConnectTransaction::Ton { messages, output_type } => Self::Ton { messages, output_type },
            WcWalletConnectTransaction::Bitcoin { data, output_type } => Self::Bitcoin { data, output_type },
            WcWalletConnectTransaction::Tron { data, output_type } => Self::Tron { data, output_type },
        }
    }
}

impl From<WcWalletConnectResponseType> for WalletConnectResponseType {
    fn from(r: WcWalletConnectResponseType) -> Self {
        match r {
            WcWalletConnectResponseType::String { value } => Self::String { value },
            WcWalletConnectResponseType::Object { json } => Self::Object { json },
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

    pub fn encode_sign_all_transactions(&self, signed_transactions: Vec<String>) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_sign_all_transactions(signed_transactions).into()
    }

    pub fn encode_send_transaction(&self, chain: Chain, transaction_id: String) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_send_transaction(chain.chain_type(), transaction_id).into()
    }

    pub fn decode_sign_message(&self, chain: Chain, sign_type: SignDigestType, data: String) -> SignMessage {
        simulation::decode_message(chain, sign_type, data)
    }

    pub fn config_session_properties(&self, properties: HashMap<String, String>, chains: Vec<String>) -> HashMap<String, String> {
        let chains = parse_chains(&chains);
        config_session_properties(properties, &chains)
    }

    pub fn decode_send_transaction(&self, transaction_type: WalletConnectTransactionType, data: String) -> Result<WalletConnectTransaction, GemstoneError> {
        let wc_type: WcWalletConnectTransactionType = transaction_type.into();
        let wc_result = WalletConnectRequestHandler::decode_send_transaction(wc_type, data).map_err(|e| GemstoneError::AnyError { msg: e })?;
        Ok(wc_result.into())
    }

    pub fn simulate_sign_message(&self, chain: Chain, sign_type: SignDigestType, data: String, session_domain: String) -> SimulationResult {
        simulation::simulate_sign_message(chain, sign_type, data, session_domain)
    }

    pub fn simulate_send_transaction(&self, chain: Chain, transaction_type: WalletConnectTransactionType, data: String) -> SimulationResult {
        simulation::simulate_send_transaction(chain, transaction_type, data)
    }
}

#[cfg(test)]
mod tests {
    use crate::message::sign_type::SignDigestType;
    use primitives::{Chain, SimulationWarning, SimulationWarningType};

    use super::WalletConnect;

    #[test]
    fn permit2_sign_message_simulation_matches_permit_warning_behavior() {
        let data = include_str!("../../../crates/gem_evm/testdata/uniswap_permit2.json").to_string();
        let result = WalletConnect::new().simulate_sign_message(Chain::Ethereum, SignDigestType::Eip712, data, "thepoc.xyz".to_string());

        assert_eq!(result.warnings.len(), 1);
        assert!(matches!(
            result.warnings.first(),
            Some(SimulationWarning {
                warning: SimulationWarningType::PermitApproval { value, .. },
                ..
            }) if value.is_none()
        ));
    }
}
