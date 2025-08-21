use crate::gateway::{GemAsset, GemTransactionLoadMetadata};
use num_bigint::BigInt;
use primitives::transaction_load::FeeOption;
use primitives::{
    GasPriceType, TransactionChange, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionMetadata,
    TransactionPerpetualMetadata, TransactionStateRequest, TransactionUpdate,
};
use std::collections::HashMap;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionUpdate {
    pub state: String,
    pub changes: Vec<GemTransactionChange>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransactionChange {
    HashChange { old: String, new: String },
    Metadata(GemTransactionMetadata),
    BlockNumber(String),
    NetworkFee(String),
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransactionMetadata {
    Perpetual(GemTransactionPerpetualMetadata),
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionPerpetualMetadata {
    pub pnl: f64,
    pub price: f64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionStateRequest {
    pub id: String,
    pub sender_address: String,
    pub created_at: i64,
    pub block_number: i64,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemStakeOperation {
    Delegate {
        validator_address: String,
    },
    Undelegate {
        validator_address: String,
    },
    Redelegate {
        src_validator_address: String,
        dst_validator_address: String,
    },
    WithdrawRewards {
        validator_addresses: Vec<String>,
    },
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransactionInputType {
    Transfer { asset: GemAsset },
    Swap { from_asset: GemAsset, to_asset: GemAsset },
    Stake { asset: GemAsset, operation: GemStakeOperation },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionLoadInput {
    pub input_type: GemTransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
    pub value: String,
    pub gas_price: crate::gateway::models::GemGasPriceType,
    pub memo: Option<String>,
    pub is_max_value: bool,
    pub metadata: GemTransactionLoadMetadata,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, uniffi::Enum)]
pub enum GemFeeOption {
    TokenAccountCreation,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemFeeOptions {
    pub options: HashMap<GemFeeOption, String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionLoadFee {
    pub fee: String,
    pub gas_price: String,
    pub gas_limit: String,
    pub options: GemFeeOptions,
}


#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionData {
    pub fee: GemTransactionLoadFee,
    pub metadata: GemTransactionLoadMetadata,
}

impl From<TransactionChange> for GemTransactionChange {
    fn from(value: TransactionChange) -> Self {
        match value {
            TransactionChange::HashChange { old, new } => GemTransactionChange::HashChange { old, new },
            TransactionChange::Metadata(metadata) => GemTransactionChange::Metadata(metadata.into()),
            TransactionChange::BlockNumber(block_number) => GemTransactionChange::BlockNumber(block_number),
            TransactionChange::NetworkFee(fee) => GemTransactionChange::NetworkFee(fee),
        }
    }
}

impl From<TransactionMetadata> for GemTransactionMetadata {
    fn from(value: TransactionMetadata) -> Self {
        match value {
            TransactionMetadata::Perpetual(perp) => GemTransactionMetadata::Perpetual(perp.into()),
        }
    }
}

impl From<TransactionPerpetualMetadata> for GemTransactionPerpetualMetadata {
    fn from(value: TransactionPerpetualMetadata) -> Self {
        GemTransactionPerpetualMetadata {
            pnl: value.pnl,
            price: value.price,
        }
    }
}

impl From<GemTransactionStateRequest> for TransactionStateRequest {
    fn from(value: GemTransactionStateRequest) -> Self {
        TransactionStateRequest {
            id: value.id,
            sender_address: value.sender_address,
            created_at: value.created_at,
            block_number: value.block_number,
        }
    }
}

impl From<TransactionUpdate> for GemTransactionUpdate {
    fn from(value: TransactionUpdate) -> Self {
        GemTransactionUpdate {
            state: value.state.to_string(),
            changes: value.changes.into_iter().map(|change| change.into()).collect(),
        }
    }
}

impl From<GemTransactionLoadInput> for TransactionLoadInput {
    fn from(value: GemTransactionLoadInput) -> Self {
        TransactionLoadInput {
            input_type: value.input_type.into(),
            sender_address: value.sender_address,
            destination_address: value.destination_address,
            value: value.value,
            gas_price: value.gas_price.into(),
            memo: value.memo,
            is_max_value: value.is_max_value,
            metadata: value.metadata.into(),
        }
    }
}

impl GemStakeOperation {
    pub fn into_primitives(self) -> primitives::StakeType {
        match self {
            GemStakeOperation::Delegate { validator_address } => primitives::StakeType::Delegate(validator_address),
            GemStakeOperation::Undelegate { validator_address } => primitives::StakeType::Undelegate(validator_address),
            GemStakeOperation::Redelegate {
                src_validator_address,
                dst_validator_address,
            } => primitives::StakeType::Redelegate(src_validator_address, dst_validator_address),
            GemStakeOperation::WithdrawRewards { validator_addresses } => primitives::StakeType::WithdrawRewards(validator_addresses),
        }
    }
}

impl From<GemTransactionInputType> for TransactionInputType {
    fn from(value: GemTransactionInputType) -> Self {
        match value {
            GemTransactionInputType::Transfer { asset } => TransactionInputType::Transfer(asset.into()),
            GemTransactionInputType::Swap { from_asset, to_asset } => TransactionInputType::Swap(from_asset.into(), to_asset.into()),
            GemTransactionInputType::Stake { asset, operation } => TransactionInputType::Stake(asset.into(), operation.into_primitives()),
        }
    }
}

impl From<crate::gateway::models::GemGasPriceType> for GasPriceType {
    fn from(value: crate::gateway::models::GemGasPriceType) -> Self {
        match value.priority_fee {
            Some(priority_fee) => GasPriceType::Eip1559 {
                gas_price: value.gas_price.parse().unwrap_or_default(),
                priority_fee: priority_fee.parse().unwrap_or_default(),
            },
            None => GasPriceType::Regular {
                gas_price: value.gas_price.parse().unwrap_or_default(),
            },
        }
    }
}

impl From<FeeOption> for GemFeeOption {
    fn from(value: FeeOption) -> Self {
        match value {
            FeeOption::TokenAccountCreation => GemFeeOption::TokenAccountCreation,
        }
    }
}

impl From<GemFeeOption> for FeeOption {
    fn from(value: GemFeeOption) -> Self {
        match value {
            GemFeeOption::TokenAccountCreation => FeeOption::TokenAccountCreation,
        }
    }
}

impl GemFeeOptions {
    pub fn new() -> Self {
        GemFeeOptions { options: HashMap::new() }
    }

    pub fn with_option(mut self, option: GemFeeOption, value: String) -> Self {
        self.options.insert(option, value);
        self
    }

    pub fn get(&self, option: &GemFeeOption) -> Option<&String> {
        self.options.get(option)
    }

    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    pub fn from_primitives(options: HashMap<FeeOption, BigInt>) -> Self {
        GemFeeOptions {
            options: options.into_iter().map(|(key, value)| (key.into(), value.to_string())).collect(),
        }
    }
}

impl From<GemTransactionLoadFee> for TransactionFee {
    fn from(value: GemTransactionLoadFee) -> Self {
        TransactionFee {
            fee: value.fee.parse().unwrap_or_default(),
            gas_price: value.gas_price.parse().unwrap_or_default(),
            gas_limit: value.gas_limit.parse().unwrap_or_default(),
            options: value
                .options
                .options
                .into_iter()
                .map(|(key, value)| {
                    let fee_option = match key {
                        GemFeeOption::TokenAccountCreation => FeeOption::TokenAccountCreation,
                    };
                    (fee_option, value.parse().unwrap_or_default())
                })
                .collect(),
        }
    }
}

impl From<TransactionFee> for GemTransactionLoadFee {
    fn from(value: TransactionFee) -> Self {
        GemTransactionLoadFee {
            fee: value.fee.to_string(),
            gas_price: value.gas_price.to_string(),
            gas_limit: value.gas_limit.to_string(),
            options: GemFeeOptions::from_primitives(value.options),
        }
    }
}

