pub mod asset;
pub mod balances;
pub mod perpetual;
pub mod staking;
pub mod transaction;

pub use balances::*;
pub use perpetual::*;
pub use staking::*;
pub use transaction::*;

// Re-export simpler models inline
use primitives::{FeePriorityValue, TransactionPreload, TransactionPreloadInput, UTXO};

// ChainAccount models
#[derive(Debug, Clone, uniffi::Record)]
pub struct GemUTXO {
    pub transaction_id: String,
    pub vout: u32,
    pub value: String,
    pub address: String,
}

// ChainState models
#[derive(Debug, Clone, uniffi::Record)]
pub struct GemFeePriorityValue {
    pub priority: String,
    pub value: String,
}

// ChainPreload models
#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionPreloadInput {
    pub sender_address: String,
    pub destination_address: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionPreload {
    pub block_hash: String,
    pub block_number: i64,
    pub utxos: Vec<GemUTXO>,
    pub sequence: u64,
    pub chain_id: String,
    pub is_destination_address_exist: bool,
}

// Conversion implementations
impl From<UTXO> for GemUTXO {
    fn from(utxo: UTXO) -> Self {
        Self {
            transaction_id: utxo.transaction_id,
            vout: utxo.vout as u32,
            value: utxo.value,
            address: utxo.address,
        }
    }
}

impl From<GemUTXO> for UTXO {
    fn from(utxo: GemUTXO) -> Self {
        Self {
            transaction_id: utxo.transaction_id,
            vout: utxo.vout as i32,
            value: utxo.value,
            address: utxo.address,
        }
    }
}

impl From<FeePriorityValue> for GemFeePriorityValue {
    fn from(fee: FeePriorityValue) -> Self {
        Self {
            priority: fee.priority.as_ref().to_string(),
            value: fee.value,
        }
    }
}

impl From<TransactionPreloadInput> for GemTransactionPreloadInput {
    fn from(input: TransactionPreloadInput) -> Self {
        Self {
            sender_address: input.sender_address,
            destination_address: input.destination_address,
        }
    }
}

impl From<GemTransactionPreloadInput> for TransactionPreloadInput {
    fn from(input: GemTransactionPreloadInput) -> Self {
        Self {
            sender_address: input.sender_address,
            destination_address: input.destination_address,
        }
    }
}

impl From<TransactionPreload> for GemTransactionPreload {
    fn from(preload: TransactionPreload) -> Self {
        Self {
            block_hash: preload.block_hash,
            block_number: preload.block_number,
            utxos: preload.utxos.into_iter().map(GemUTXO::from).collect(),
            sequence: preload.sequence,
            chain_id: preload.chain_id,
            is_destination_address_exist: preload.is_destination_address_exist,
        }
    }
}
