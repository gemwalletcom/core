use crate::gateway::models::asset::GemAsset;
use crate::gateway::models::GemUTXO;
use primitives::{
    GasPrice, TransactionChange, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionMetadata, TransactionPerpetualMetadata,
    TransactionStateRequest, TransactionUpdate,
};
use primitives::transaction_load::TransactionLoadMetadata;
use primitives::transaction_load::FeeOption;
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
        asset: GemAsset,
        validator_address: String,
    },
    Undelegate {
        asset: GemAsset,
        validator_address: String,
    },
    Redelegate {
        asset: GemAsset,
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
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, uniffi::Enum)]
pub enum GemFeeOption {
    TokenAccountCreation,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionLoadFee {
    pub fee: String,
    pub gas_price: String,
    pub gas_limit: String,
    pub options: HashMap<GemFeeOption, String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSignerInputToken {
    pub sender_token_address: String,
    pub recipient_token_address: Option<String>,
    pub token_program: String,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransactionLoadMetadata {
    Solana {
        sender_token_address: String,
        recipient_token_address: Option<String>,
        token_program: String,
        sequence: i64,
    },
    Ton {
        jetton_wallet_address: String,
        sequence: i64,
    },
    Cosmos {
        account_number: i64,
        sequence: i64,
        chain_id: String,
    },
    Bitcoin {
        utxos: Vec<GemUTXO>,
    },
    Cardano {
        utxos: Vec<GemUTXO>,
    },
    Evm {
        chain_id: String,
        block_hash: String,
        block_number: i64,
    },
    Near {
        sequence: i64,
        block_hash: String,
        is_destination_exist: bool,
    },
    Stellar {
        sequence: i64,
    },
    Xrp {
        sequence: i64,
    },
    Algorand {
        sequence: i64,
    },
    Aptos {
        sequence: i64,
    },
    Polkadot {
        sequence: i64,
        genesis_hash: String,
        block_hash: String,
        block_number: i64,
        spec_version: u64,
        transaction_version: u64,
        period: i64,
    },
    Tron {
        block_number: i64,
        block_version: i64,
        block_timestamp: i64,
        transaction_tree_root: String,
        parent_hash: String,
        witness_address: String,
    },
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
            sequence: value.sequence,
            block_hash: value.block_hash,
            block_number: value.block_number as u64,
            chain_id: value.chain_id,
            utxos: value.utxos.into_iter().map(|utxo| utxo.into()).collect(),
            memo: value.memo,
        }
    }
}

impl From<GemStakeOperation> for primitives::StakeOperation {
    fn from(value: GemStakeOperation) -> Self {
        match value {
            GemStakeOperation::Delegate { asset, validator_address } => primitives::StakeOperation::Delegate(asset.into(), validator_address),
            GemStakeOperation::Undelegate { asset, validator_address } => primitives::StakeOperation::Undelegate(asset.into(), validator_address),
            GemStakeOperation::Redelegate {
                asset,
                src_validator_address,
                dst_validator_address,
            } => primitives::StakeOperation::Redelegate(asset.into(), src_validator_address, dst_validator_address),
            GemStakeOperation::WithdrawRewards { validator_addresses } => primitives::StakeOperation::WithdrawRewards(validator_addresses),
        }
    }
}

impl From<GemTransactionInputType> for TransactionInputType {
    fn from(value: GemTransactionInputType) -> Self {
        match value {
            GemTransactionInputType::Transfer { asset } => TransactionInputType::Transfer(asset.into()),
            GemTransactionInputType::Swap { from_asset, to_asset } => TransactionInputType::Swap(from_asset.into(), to_asset.into()),
            GemTransactionInputType::Stake { operation } => TransactionInputType::Stake(operation.into()),
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

impl From<FeeOption> for GemFeeOption {
    fn from(value: FeeOption) -> Self {
        match value {
            FeeOption::TokenAccountCreation => GemFeeOption::TokenAccountCreation,
        }
    }
}

impl From<TransactionLoadMetadata> for GemTransactionLoadMetadata {
    fn from(value: TransactionLoadMetadata) -> Self {
        match value {
            TransactionLoadMetadata::Solana {
                sender_token_address,
                recipient_token_address,
                token_program,
                sequence,
            } => GemTransactionLoadMetadata::Solana {
                sender_token_address,
                recipient_token_address,
                token_program: token_program.as_ref().to_string(),
                sequence: sequence as i64,
            },
            TransactionLoadMetadata::Ton { jetton_wallet_address, sequence } => GemTransactionLoadMetadata::Ton { 
                jetton_wallet_address,
                sequence: sequence as i64,
            },
            TransactionLoadMetadata::Cosmos { account_number, sequence, chain_id } => GemTransactionLoadMetadata::Cosmos { 
                account_number: account_number as i64,
                sequence: sequence as i64,
                chain_id,
            },
            TransactionLoadMetadata::Bitcoin { utxos } => GemTransactionLoadMetadata::Bitcoin {
                utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
            },
            TransactionLoadMetadata::Cardano { utxos } => GemTransactionLoadMetadata::Cardano {
                utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
            },
            TransactionLoadMetadata::Evm { chain_id, block_hash, block_number } => GemTransactionLoadMetadata::Evm {
                chain_id,
                block_hash,
                block_number: block_number as i64,
            },
            TransactionLoadMetadata::Near { sequence, block_hash, is_destination_exist } => GemTransactionLoadMetadata::Near {
                sequence: sequence as i64,
                block_hash,
                is_destination_exist,
            },
            TransactionLoadMetadata::Stellar { sequence } => GemTransactionLoadMetadata::Stellar {
                sequence: sequence as i64,
            },
            TransactionLoadMetadata::Xrp { sequence } => GemTransactionLoadMetadata::Xrp {
                sequence: sequence as i64,
            },
            TransactionLoadMetadata::Algorand { sequence } => GemTransactionLoadMetadata::Algorand {
                sequence: sequence as i64,
            },
            TransactionLoadMetadata::Aptos { sequence } => GemTransactionLoadMetadata::Aptos {
                sequence: sequence as i64,
            },
            TransactionLoadMetadata::Polkadot { 
                sequence, 
                genesis_hash, 
                block_hash, 
                block_number, 
                spec_version, 
                transaction_version, 
                period 
            } => GemTransactionLoadMetadata::Polkadot {
                sequence: sequence as i64,
                genesis_hash,
                block_hash,
                block_number: block_number as i64,
                spec_version,
                transaction_version,
                period: period as i64,
            },
            TransactionLoadMetadata::Tron {
                block_number,
                block_version,
                block_timestamp,
                transaction_tree_root,
                parent_hash,
                witness_address,
            } => GemTransactionLoadMetadata::Tron {
                block_number: block_number as i64,
                block_version: block_version as i64,
                block_timestamp: block_timestamp as i64,
                transaction_tree_root,
                parent_hash,
                witness_address,
            },
        }
    }
}


pub fn map_transaction_load_data(load_data: TransactionLoadData, _input: &GemTransactionLoadInput) -> GemTransactionData {
    GemTransactionData {
        fee: GemTransactionLoadFee {
            fee: load_data.fee.fee.to_string(),
            gas_price: load_data.fee.gas_price.to_string(),
            gas_limit: load_data.fee.gas_limit.to_string(),
            options: load_data.fee.options.into_iter().map(|(key, value)| (key.into(), value)).collect(),
        },
        metadata: load_data.metadata.into(),
    }
}
