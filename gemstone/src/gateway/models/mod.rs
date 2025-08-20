pub mod asset;
pub mod balances;
pub mod perpetual;
pub mod staking;
pub mod transaction;
pub mod transaction_metadata;

pub use balances::*;
pub use perpetual::*;
pub use staking::*;
pub use transaction::*;
pub use transaction_metadata::*;

// Re-export simpler models inline
use primitives::{FeeRate, TransactionPreloadInput, UTXO};

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
pub struct GemGasPriceType {
    pub gas_price: String,
    pub priority_fee: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemFeeRate {
    pub priority: String,
    pub gas_price_type: GemGasPriceType,
}

// ChainPreload models
#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionPreloadInput {
    pub sender_address: String,
    pub destination_address: String,
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

impl From<FeeRate> for GemFeeRate {
    fn from(fee: FeeRate) -> Self {
        let gas_price_type = match fee.gas_price_type {
            primitives::GasPriceType::Regular { gas_price } => GemGasPriceType {
                gas_price: gas_price.to_string(),
                priority_fee: None,
            },
            primitives::GasPriceType::Eip1559 { gas_price, priority_fee } => GemGasPriceType {
                gas_price: gas_price.to_string(),
                priority_fee: Some(priority_fee.to_string()),
            },
        };
        
        Self {
            priority: fee.priority.as_ref().to_string(),
            gas_price_type,
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

