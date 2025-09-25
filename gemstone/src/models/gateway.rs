use crate::models::GemTransactionInputType;
use primitives::{BroadcastOptions, FeeRate, GasPriceType, TransactionPreloadInput, UTXO};

pub type GemUTXO = UTXO;

#[uniffi::remote(Record)]
pub struct GemUTXO {
    pub transaction_id: String,
    pub vout: i32,
    pub value: String,
    pub address: String,
}

pub type GemBroadcastOptions = BroadcastOptions;

#[uniffi::remote(Record)]
pub struct BroadcastOptions {
    pub skip_preflight: bool,
}

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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionPreloadInput {
    pub input_type: GemTransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
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
