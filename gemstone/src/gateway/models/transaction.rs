use crate::gateway::{GemAsset, GemDelegation, GemDelegationValidator, GemGasPriceType, GemTransactionLoadMetadata};
use num_bigint::BigInt;
use primitives::transaction_load::FeeOption;
use primitives::{
    GasPriceType, StakeType, TransactionChange, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionMetadata, TransactionPerpetualMetadata,
    TransactionStateRequest, TransactionUpdate, WalletConnectionSessionAppMetadata, TransferDataExtra, TransferDataOutputType,
};
use primitives::swap::ApprovalData;
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
pub enum GemStakeType {
    Delegate {
        validator: GemDelegationValidator,
    },
    Undelegate {
        delegation: GemDelegation,
    },
    Redelegate {
        delegation: GemDelegation,
        to_validator: GemDelegationValidator,
    },
    WithdrawRewards {
        validators: Vec<GemDelegationValidator>,
    },
    Withdraw {
        delegation: GemDelegation,
    },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletConnectionSessionAppMetadata {
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon: String,
    pub redirect_native: Option<String>,
    pub redirect_universal: Option<String>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransferDataOutputType {
    EncodedTransaction,
    Signature,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransferDataExtra {
    pub gas_limit: Option<String>,
    pub gas_price: Option<GemGasPriceType>,
    pub data: Option<Vec<u8>>,
    pub output_type: GemTransferDataOutputType,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemTransactionInputType {
    Transfer { asset: GemAsset },
    Deposit { asset: GemAsset },
    Swap { from_asset: GemAsset, to_asset: GemAsset },
    Stake { asset: GemAsset, stake_type: GemStakeType },
    TokenApprove { asset: GemAsset, approval_data: GemApprovalData },
    Generic { asset: GemAsset, metadata: GemWalletConnectionSessionAppMetadata, extra: GemTransferDataExtra },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionLoadInput {
    pub input_type: GemTransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
    pub value: String,
    pub gas_price: GemGasPriceType,
    pub memo: Option<String>,
    pub is_max_value: bool,
    pub metadata: GemTransactionLoadMetadata,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, uniffi::Enum)]
pub enum GemFeeOption {
    TokenAccountCreation,
}

#[derive(Debug, Default, Clone, uniffi::Record)]
pub struct GemFeeOptions {
    pub options: HashMap<GemFeeOption, String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionLoadFee {
    pub fee: String,
    pub gas_price_type: GemGasPriceType,
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
            TransactionChange::NetworkFee(fee) => GemTransactionChange::NetworkFee(fee.to_string()),
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

impl From<GemStakeType> for StakeType {
    fn from(value: GemStakeType) -> Self {
        match value {
            GemStakeType::Delegate { validator } => StakeType::Delegate(validator.into()),
            GemStakeType::Undelegate { delegation } => StakeType::Undelegate(delegation.into()),
            GemStakeType::Redelegate { delegation, to_validator } => StakeType::Redelegate(delegation.into(), to_validator.into()),
            GemStakeType::WithdrawRewards { validators } => StakeType::WithdrawRewards(validators.into_iter().map(|v| v.into()).collect()),
            GemStakeType::Withdraw { delegation } => StakeType::Withdraw(delegation.into()),
        }
    }
}

impl From<GemWalletConnectionSessionAppMetadata> for WalletConnectionSessionAppMetadata {
    fn from(value: GemWalletConnectionSessionAppMetadata) -> Self {
        WalletConnectionSessionAppMetadata {
            name: value.name,
            description: value.description,
            url: value.url,
            icon: value.icon,
            redirect_native: value.redirect_native,
            redirect_universal: value.redirect_universal,
        }
    }
}

impl From<GemTransferDataOutputType> for TransferDataOutputType {
    fn from(value: GemTransferDataOutputType) -> Self {
        match value {
            GemTransferDataOutputType::EncodedTransaction => TransferDataOutputType::EncodedTransaction,
            GemTransferDataOutputType::Signature => TransferDataOutputType::Signature,
        }
    }
}

impl From<GemTransferDataExtra> for TransferDataExtra {
    fn from(value: GemTransferDataExtra) -> Self {
        TransferDataExtra {
            gas_limit: value.gas_limit.map(|s| s.parse().unwrap_or_default()),
            gas_price: value.gas_price.map(|gp| gp.into()),
            data: value.data,
            output_type: value.output_type.into(),
        }
    }
}

impl From<GemApprovalData> for ApprovalData {
    fn from(value: GemApprovalData) -> Self {
        ApprovalData {
            token: value.token,
            spender: value.spender,
            value: value.value,
        }
    }
}

impl From<GemTransactionInputType> for TransactionInputType {
    fn from(value: GemTransactionInputType) -> Self {
        match value {
            GemTransactionInputType::Transfer { asset } => TransactionInputType::Transfer(asset.into()),
            GemTransactionInputType::Deposit { asset } => TransactionInputType::Deposit(asset.into()),
            GemTransactionInputType::Swap { from_asset, to_asset } => TransactionInputType::Swap(from_asset.into(), to_asset.into()),
            GemTransactionInputType::Stake { asset, stake_type: operation } => TransactionInputType::Stake(asset.into(), operation.into()),
            GemTransactionInputType::TokenApprove { asset, approval_data } => TransactionInputType::TokenApprove(asset.into(), approval_data.into()),
            GemTransactionInputType::Generic { asset, metadata, extra } => TransactionInputType::Generic(asset.into(), metadata.into(), extra.into()),
        }
    }
}

impl From<GemGasPriceType> for GasPriceType {
    fn from(value: GemGasPriceType) -> Self {
        match value {
            GemGasPriceType::Regular { gas_price } => GasPriceType::Regular {
                gas_price: gas_price.parse().unwrap_or_default(),
            },
            GemGasPriceType::Eip1559 { gas_price, priority_fee } => GasPriceType::Eip1559 {
                gas_price: gas_price.parse().unwrap_or_default(),
                priority_fee: priority_fee.parse().unwrap_or_default(),
            },
            GemGasPriceType::Solana {
                gas_price,
                priority_fee,
                unit_price,
            } => GasPriceType::Solana {
                gas_price: gas_price.parse().unwrap_or_default(),
                priority_fee: priority_fee.parse().unwrap_or_default(),
                unit_price: unit_price.parse().unwrap_or_default(),
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
            gas_price_type: value.gas_price_type.into(),
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
            gas_price_type: value.gas_price_type.into(),
            gas_limit: value.gas_limit.to_string(),
            options: GemFeeOptions::from_primitives(value.options),
        }
    }
}
