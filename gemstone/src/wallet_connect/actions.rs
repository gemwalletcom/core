use crate::message::sign_type::SignDigestType;
use primitives::{Chain, TransferDataOutputType, WCEthereumTransaction};

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

impl WalletConnectTransactionType {
    pub fn get_output_type(&self) -> Option<TransferDataOutputType> {
        match self {
            Self::Ethereum => None,
            Self::Solana { output_type } | Self::Sui { output_type } | Self::Ton { output_type } | Self::Bitcoin { output_type } | Self::Tron { output_type } => {
                Some(output_type.clone())
            }
        }
    }
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
pub enum WalletConnectChainOperation {
    AddChain,
    SwitchChain,
    GetChainId,
}

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
