pub mod asset;
pub mod balances;
pub mod perpetual;
pub mod staking;
pub mod transaction;
pub mod transaction_metadata;

pub use asset::*;
pub use balances::*;
pub use perpetual::*;
pub use staking::*;
pub use transaction::*;
pub use transaction_metadata::*;

use primitives::{FeeRate, GasPriceType, TransactionPreloadInput, UTXO};

// ChainAccount models
#[derive(Debug, Clone, uniffi::Record)]
pub struct GemUTXO {
    pub transaction_id: String,
    pub vout: u32,
    pub value: String,
    pub address: String,
}

// ChainState models
#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemGasPriceType {
    Regular {
        gas_price: String,
    },
    Eip1559 {
        gas_price: String,
        priority_fee: String,
    },
    Solana {
        gas_price: String,
        priority_fee: String,
        unit_price: String,
    },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemFeeRate {
    pub priority: String,
    pub gas_price_type: GemGasPriceType,
}

// ChainPreload models
#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionPreloadInput {
    pub input_type: GemTransactionInputType,
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

impl From<GasPriceType> for GemGasPriceType {
    fn from(value: GasPriceType) -> Self {
        match value {
            GasPriceType::Regular { gas_price } => GemGasPriceType::Regular {
                gas_price: gas_price.to_string(),
            },
            GasPriceType::Eip1559 { gas_price, priority_fee } => GemGasPriceType::Eip1559 {
                gas_price: gas_price.to_string(),
                priority_fee: priority_fee.to_string(),
            },
            GasPriceType::Solana {
                gas_price,
                priority_fee,
                unit_price,
            } => GemGasPriceType::Solana {
                gas_price: gas_price.to_string(),
                priority_fee: priority_fee.to_string(),
                unit_price: unit_price.to_string(),
            },
        }
    }
}

impl From<FeeRate> for GemFeeRate {
    fn from(fee: FeeRate) -> Self {
        Self {
            priority: fee.priority.as_ref().to_string(),
            gas_price_type: fee.gas_price_type.into(),
        }
    }
}

impl From<TransactionPreloadInput> for GemTransactionPreloadInput {
    fn from(input: TransactionPreloadInput) -> Self {
        Self {
            input_type: input.input_type.into(),
            sender_address: input.sender_address,
            destination_address: input.destination_address,
        }
    }
}

impl From<GemTransactionPreloadInput> for TransactionPreloadInput {
    fn from(input: GemTransactionPreloadInput) -> Self {
        Self {
            input_type: input.input_type.into(),
            sender_address: input.sender_address,
            destination_address: input.destination_address,
        }
    }
}
