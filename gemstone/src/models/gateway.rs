use primitives::{Asset, BroadcastOptions, FeeRate, GasPriceType, TransactionInputType, TransactionPreloadInput, UTXO};

pub type GemBroadcastOptions = BroadcastOptions;
pub type GemUTXO = UTXO;

#[uniffi::remote(Record)]
pub struct GemUTXO {
    pub transaction_id: String,
    pub vout: i32,
    pub value: String,
    pub address: String,
}

#[uniffi::remote(Record)]
pub struct GemBroadcastOptions {
    pub skip_preflight: bool,
    pub from_address: Option<String>,
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
    pub input_type: String,
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
            input_type: match input.input_type {
                TransactionInputType::Transfer(_) => "transfer".to_string(),
                TransactionInputType::Deposit(_) => "deposit".to_string(),
                TransactionInputType::Swap(_, _, _) => "swap".to_string(),
                TransactionInputType::Stake(_, _) => "stake".to_string(),
                TransactionInputType::TokenApprove(_, _) => "token_approve".to_string(),
                TransactionInputType::Generic(_, _, _) => "generic".to_string(),
                TransactionInputType::Perpetual(_, _) => "perpetual".to_string(),
            },
            sender_address: input.sender_address,
            destination_address: input.destination_address,
        }
    }
}

impl From<GemTransactionPreloadInput> for TransactionPreloadInput {
    fn from(input: GemTransactionPreloadInput) -> Self {
        use primitives::Chain;
        let asset = Asset::from_chain(Chain::Ethereum);
        Self {
            input_type: TransactionInputType::Transfer(asset),
            sender_address: input.sender_address,
            destination_address: input.destination_address,
        }
    }
}
