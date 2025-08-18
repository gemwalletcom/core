use primitives::{TransactionChange, TransactionMetadata, TransactionPerpetualMetadata, TransactionStateRequest, TransactionUpdate, TransactionLoadInput, TransactionLoadData, TransactionInputType, GasPrice};
use crate::gateway::models::asset::GemAsset;
use crate::gateway::models::GemUTXO;

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
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemStakeOperation {
    Delegate { asset: GemAsset, validator_address: String },
    Undelegate { asset: GemAsset, validator_address: String },
    Redelegate { asset: GemAsset, src_validator_address: String, dst_validator_address: String },
    WithdrawRewards { validator_addresses: Vec<String> },
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransactionInputType {
    Transfer { asset: GemAsset },
    Swap { from_asset: GemAsset, to_asset: GemAsset },
    Stake { operation: GemStakeOperation },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemGasPrice {
    pub gas_price: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionLoadInput {
    pub input_type: GemTransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
    pub value: String,
    pub gas_price: GemGasPrice,
    pub sequence: u64,
    pub block_hash: String,
    pub block_number: i64,
    pub chain_id: String,
    pub utxos: Vec<GemUTXO>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionLoadFee {
    pub fee: String,
    pub gas_price: String,
    pub gas_limit: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionData {
    pub account_number: i32,
    pub sequence: i32,
    pub block_hash: String,
    pub block_number: i64,
    pub chain_id: String,
    pub fee: GemTransactionLoadFee,
    pub utxos: Vec<GemUTXO>,
    pub message_bytes: String,
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
            sequence: value.sequence,
            block_hash: value.block_hash,
            block_number: value.block_number,
            chain_id: value.chain_id,
            utxos: value.utxos.into_iter().map(|utxo| utxo.into()).collect(),
        }
    }
}

impl From<GemStakeOperation> for primitives::StakeOperation {
    fn from(value: GemStakeOperation) -> Self {
        match value {
            GemStakeOperation::Delegate { asset, validator_address } => {
                primitives::StakeOperation::Delegate(asset.into(), validator_address)
            }
            GemStakeOperation::Undelegate { asset, validator_address } => {
                primitives::StakeOperation::Undelegate(asset.into(), validator_address)
            }
            GemStakeOperation::Redelegate { asset, src_validator_address, dst_validator_address } => {
                primitives::StakeOperation::Redelegate(asset.into(), src_validator_address, dst_validator_address)
            }
            GemStakeOperation::WithdrawRewards { validator_addresses } => {
                primitives::StakeOperation::WithdrawRewards(validator_addresses)
            }
        }
    }
}

impl From<GemTransactionInputType> for TransactionInputType {
    fn from(value: GemTransactionInputType) -> Self {
        match value {
            GemTransactionInputType::Transfer { asset } => TransactionInputType::Transfer(asset.into()),
            GemTransactionInputType::Swap { from_asset, to_asset } => {
                TransactionInputType::Swap(from_asset.into(), to_asset.into())
            }
            GemTransactionInputType::Stake { operation } => {
                TransactionInputType::Stake(operation.into())
            }
        }
    }
}

impl From<GemGasPrice> for GasPrice {
    fn from(value: GemGasPrice) -> Self {
        GasPrice {
            gas_price: value.gas_price.parse().unwrap_or_default(),
        }
    }
}

pub fn map_transaction_load_data(load_data: TransactionLoadData, input: &GemTransactionLoadInput) -> GemTransactionData {
    GemTransactionData {
        account_number: load_data.account_number as i32,
        sequence: load_data.sequence as i32,
        block_hash: input.block_hash.clone(),
        block_number: input.block_number,
        chain_id: input.chain_id.clone(),
        fee: GemTransactionLoadFee {
            fee: load_data.fee.fee.to_string(),
            gas_price: load_data.fee.gas_price.to_string(),
            gas_limit: load_data.fee.gas_limit.to_string(),
        },
        utxos: input.utxos.clone(),
        message_bytes: "".to_string(),
    }
}

