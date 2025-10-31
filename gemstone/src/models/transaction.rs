use crate::models::*;
use num_bigint::BigInt;
use primitives::stake_type::{FreezeData, StakeData};
use primitives::{
    AccountDataType, Asset, FeeOption, GasPriceType, HyperliquidOrder, PerpetualConfirmData, PerpetualDirection, PerpetualProvider, PerpetualReduceData,
    PerpetualType, StakeType, TransactionChange, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, TransactionMetadata,
    TransactionPerpetualMetadata, TransactionState, TransactionStateRequest, TransactionUpdate, TransferDataExtra, TransferDataOutputAction,
    TransferDataOutputType, WalletConnectionSessionAppMetadata,
};
use std::collections::HashMap;
use swap::{GemApprovalData, GemSwapData};

pub type GemPerpetualDirection = PerpetualDirection;
pub type GemPerpetualProvider = PerpetualProvider;
pub type GemPerpetualConfirmData = PerpetualConfirmData;
pub type GemPerpetualReduceData = PerpetualReduceData;
pub type GemFeeOption = FeeOption;
pub type GemTransferDataOutputType = TransferDataOutputType;
pub type GemTransferDataOutputAction = TransferDataOutputAction;
pub type GemTransactionPerpetualMetadata = TransactionPerpetualMetadata;
pub type GemTransactionMetadata = TransactionMetadata;
pub type GemTransactionState = TransactionState;
pub type GemTransactionChange = TransactionChange;
pub type GemTransactionUpdate = TransactionUpdate;

#[uniffi::remote(Enum)]
pub enum PerpetualDirection {
    Short,
    Long,
}

#[uniffi::remote(Enum)]
pub enum PerpetualProvider {
    Hypercore,
}

#[uniffi::remote(Enum)]
pub enum FeeOption {
    TokenAccountCreation,
}

#[uniffi::remote(Enum)]
pub enum TransferDataOutputType {
    EncodedTransaction,
    Signature,
}

#[uniffi::remote(Enum)]
pub enum TransferDataOutputAction {
    Sign,
    Send,
}

#[uniffi::remote(Record)]
pub struct TransactionPerpetualMetadata {
    pub pnl: f64,
    pub price: f64,
    pub direction: PerpetualDirection,
    pub provider: Option<PerpetualProvider>,
}

#[uniffi::remote(Enum)]
pub enum TransactionMetadata {
    Perpetual(TransactionPerpetualMetadata),
}

#[uniffi::remote(Enum)]
pub enum TransactionState {
    Pending,
    Confirmed,
    Failed,
    Reverted,
}

#[uniffi::remote(Enum)]
pub enum TransactionChange {
    HashChange { old: String, new: String },
    Metadata(TransactionMetadata),
    BlockNumber(String),
    NetworkFee(BigInt),
}

#[uniffi::remote(Record)]
pub struct TransactionUpdate {
    pub state: TransactionState,
    pub changes: Vec<TransactionChange>,
}

pub type GemAccountDataType = AccountDataType;

#[uniffi::remote(Enum)]
pub enum GemAccountDataType {
    Activate,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionStateRequest {
    pub id: String,
    pub sender_address: String,
    pub created_at: i64,
    pub block_number: i64,
}

pub type GemHyperliquidOrder = HyperliquidOrder;

pub type GemStakeData = StakeData;

#[uniffi::remote(Record)]
pub struct GemStakeData {
    pub data: Option<String>,
    pub to: Option<String>,
}

#[uniffi::remote(Record)]
pub struct GemHyperliquidOrder {
    pub approve_agent_required: bool,
    pub approve_referral_required: bool,
    pub approve_builder_required: bool,
    pub builder_fee_bps: u32,
    pub agent_address: String,
    pub agent_private_key: String,
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
    Freeze {
        freeze_data: GemFreezeData,
    },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemFreezeData {
    pub freeze_type: GemFreezeType,
    pub resource: GemResource,
}

pub type GemWalletConnectionSessionAppMetadata = WalletConnectionSessionAppMetadata;

#[uniffi::remote(Record)]
pub struct GemWalletConnectionSessionAppMetadata {
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransferDataExtra {
    pub to: String,
    pub gas_limit: Option<String>,
    pub gas_price: Option<GemGasPriceType>,
    pub data: Option<Vec<u8>>,
    pub output_type: GemTransferDataOutputType,
    pub output_action: GemTransferDataOutputAction,
}

#[uniffi::remote(Record)]
pub struct PerpetualConfirmData {
    pub direction: PerpetualDirection,
    pub base_asset: Asset,
    pub asset_index: i32,
    pub price: String,
    pub fiat_value: f64,
    pub size: String,
    pub slippage: f64,
    pub leverage: u8,
    pub pnl: Option<f64>,
    pub entry_price: Option<f64>,
    pub market_price: f64,
    pub margin_amount: f64,
}

#[uniffi::remote(Record)]
pub struct PerpetualReduceData {
    pub data: PerpetualConfirmData,
    pub position_direction: PerpetualDirection,
}

pub type GemPerpetualType = PerpetualType;

#[uniffi::remote(Enum)]
pub enum PerpetualType {
    Open(PerpetualConfirmData),
    Close(PerpetualConfirmData),
    Increase(PerpetualConfirmData),
    Reduce(PerpetualReduceData),
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemTransactionInputType {
    Transfer {
        asset: GemAsset,
    },
    Deposit {
        asset: GemAsset,
    },
    Swap {
        from_asset: GemAsset,
        to_asset: GemAsset,
        swap_data: GemSwapData,
    },
    Stake {
        asset: GemAsset,
        stake_type: GemStakeType,
    },
    TokenApprove {
        asset: GemAsset,
        approval_data: GemApprovalData,
    },
    Generic {
        asset: GemAsset,
        metadata: GemWalletConnectionSessionAppMetadata,
        extra: GemTransferDataExtra,
    },
    TransferNft {
        asset: GemAsset,
        nft_asset: GemNFTAsset,
    },
    Account {
        asset: GemAsset,
        account_type: GemAccountDataType,
    },
    Perpetual {
        asset: GemAsset,
        perpetual_type: GemPerpetualType,
    },
}

impl GemTransactionInputType {
    pub fn asset(&self) -> &GemAsset {
        match self {
            Self::Transfer { asset }
            | Self::Deposit { asset }
            | Self::Stake { asset, .. }
            | Self::TokenApprove { asset, .. }
            | Self::Generic { asset, .. }
            | Self::TransferNft { asset, .. }
            | Self::Account { asset, .. }
            | Self::Perpetual { asset, .. } => asset,
            Self::Swap { from_asset, .. } => from_asset,
        }
    }

    pub fn swap_data(&self) -> Result<&GemSwapData, String> {
        match self {
            Self::Swap { swap_data, .. } => Ok(swap_data),
            _ => Err("Expected Swap".to_string()),
        }
    }

    pub fn stake_type(&self) -> Result<&GemStakeType, String> {
        match self {
            Self::Stake { stake_type, .. } => Ok(stake_type),
            _ => Err("Expected Stake".to_string()),
        }
    }

    pub fn perpetual_type(&self) -> Result<&GemPerpetualType, String> {
        match self {
            Self::Perpetual { perpetual_type, .. } => Ok(perpetual_type),
            _ => Err("Expected Perpetual".to_string()),
        }
    }
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

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransactionLoadMetadata {
    None,
    Solana {
        sender_token_address: Option<String>,
        recipient_token_address: Option<String>,
        token_program: Option<GemSolanaTokenProgramId>,
        block_hash: String,
    },
    Ton {
        sender_token_address: Option<String>,
        recipient_token_address: Option<String>,
        sequence: u64,
    },
    Cosmos {
        account_number: u64,
        sequence: u64,
        chain_id: String,
    },
    Bitcoin {
        utxos: Vec<GemUTXO>,
    },
    Zcash {
        utxos: Vec<GemUTXO>,
        branch_id: String,
    },
    Cardano {
        utxos: Vec<GemUTXO>,
    },
    Evm {
        nonce: u64,
        chain_id: u64,
        stake_data: Option<GemStakeData>,
    },
    Near {
        sequence: u64,
        block_hash: String,
    },
    Stellar {
        sequence: u64,
        is_destination_address_exist: bool,
    },
    Xrp {
        sequence: u64,
        block_number: u64,
    },
    Algorand {
        sequence: u64,
        block_hash: String,
        chain_id: String,
    },
    Aptos {
        sequence: u64,
        data: Option<String>,
    },
    Polkadot {
        sequence: u64,
        genesis_hash: String,
        block_hash: String,
        block_number: u64,
        spec_version: u64,
        transaction_version: u64,
        period: u64,
    },
    Tron {
        block_number: u64,
        block_version: u64,
        block_timestamp: u64,
        transaction_tree_root: String,
        parent_hash: String,
        witness_address: String,
        votes: HashMap<String, u64>,
    },
    Sui {
        message_bytes: String,
    },
    Hyperliquid {
        order: Option<GemHyperliquidOrder>,
    },
}

impl From<TransactionLoadMetadata> for GemTransactionLoadMetadata {
    fn from(value: TransactionLoadMetadata) -> Self {
        match value {
            TransactionLoadMetadata::None => GemTransactionLoadMetadata::None,
            TransactionLoadMetadata::Solana {
                sender_token_address,
                recipient_token_address,
                token_program,
                block_hash,
            } => GemTransactionLoadMetadata::Solana {
                sender_token_address,
                recipient_token_address,
                token_program,
                block_hash,
            },
            TransactionLoadMetadata::Ton {
                sender_token_address,
                recipient_token_address,
                sequence,
            } => GemTransactionLoadMetadata::Ton {
                sender_token_address,
                recipient_token_address,
                sequence,
            },
            TransactionLoadMetadata::Cosmos {
                account_number,
                sequence,
                chain_id,
            } => GemTransactionLoadMetadata::Cosmos {
                account_number,
                sequence,
                chain_id,
            },
            TransactionLoadMetadata::Bitcoin { utxos } => GemTransactionLoadMetadata::Bitcoin { utxos },
            TransactionLoadMetadata::Zcash { utxos, branch_id } => GemTransactionLoadMetadata::Zcash { utxos, branch_id },
            TransactionLoadMetadata::Cardano { utxos } => GemTransactionLoadMetadata::Cardano { utxos },
            TransactionLoadMetadata::Evm { nonce, chain_id, stake_data } => GemTransactionLoadMetadata::Evm { nonce, chain_id, stake_data },
            TransactionLoadMetadata::Near { sequence, block_hash } => GemTransactionLoadMetadata::Near { sequence, block_hash },
            TransactionLoadMetadata::Stellar {
                sequence,
                is_destination_address_exist,
            } => GemTransactionLoadMetadata::Stellar {
                sequence,
                is_destination_address_exist,
            },
            TransactionLoadMetadata::Xrp { sequence, block_number } => GemTransactionLoadMetadata::Xrp { sequence, block_number },
            TransactionLoadMetadata::Algorand {
                sequence,
                block_hash,
                chain_id,
            } => GemTransactionLoadMetadata::Algorand {
                sequence,
                block_hash,
                chain_id,
            },
            TransactionLoadMetadata::Aptos { sequence, data } => GemTransactionLoadMetadata::Aptos { sequence, data },
            TransactionLoadMetadata::Polkadot {
                sequence,
                genesis_hash,
                block_hash,
                block_number,
                spec_version,
                transaction_version,
                period,
            } => GemTransactionLoadMetadata::Polkadot {
                sequence,
                genesis_hash,
                block_hash,
                block_number,
                spec_version,
                transaction_version,
                period,
            },
            TransactionLoadMetadata::Tron {
                block_number,
                block_version,
                block_timestamp,
                transaction_tree_root,
                parent_hash,
                witness_address,
                votes,
            } => GemTransactionLoadMetadata::Tron {
                block_number,
                block_version,
                block_timestamp,
                transaction_tree_root,
                parent_hash,
                witness_address,
                votes,
            },
            TransactionLoadMetadata::Sui { message_bytes } => GemTransactionLoadMetadata::Sui { message_bytes },
            TransactionLoadMetadata::Hyperliquid { order } => GemTransactionLoadMetadata::Hyperliquid { order },
        }
    }
}

impl From<GemTransactionLoadMetadata> for TransactionLoadMetadata {
    fn from(value: GemTransactionLoadMetadata) -> Self {
        match value {
            GemTransactionLoadMetadata::None => TransactionLoadMetadata::None,
            GemTransactionLoadMetadata::Solana {
                sender_token_address,
                recipient_token_address,
                token_program,
                block_hash,
            } => TransactionLoadMetadata::Solana {
                sender_token_address,
                recipient_token_address,
                token_program,
                block_hash,
            },
            GemTransactionLoadMetadata::Ton {
                sender_token_address,
                recipient_token_address,
                sequence,
            } => TransactionLoadMetadata::Ton {
                sender_token_address,
                recipient_token_address,
                sequence,
            },
            GemTransactionLoadMetadata::Cosmos {
                account_number,
                sequence,
                chain_id,
            } => TransactionLoadMetadata::Cosmos {
                account_number,
                sequence,
                chain_id,
            },
            GemTransactionLoadMetadata::Bitcoin { utxos } => TransactionLoadMetadata::Bitcoin { utxos },
            GemTransactionLoadMetadata::Zcash { utxos, branch_id } => TransactionLoadMetadata::Zcash { utxos, branch_id },
            GemTransactionLoadMetadata::Cardano { utxos } => TransactionLoadMetadata::Cardano { utxos },
            GemTransactionLoadMetadata::Evm { nonce, chain_id, stake_data } => TransactionLoadMetadata::Evm { nonce, chain_id, stake_data },
            GemTransactionLoadMetadata::Near { sequence, block_hash } => TransactionLoadMetadata::Near { sequence, block_hash },
            GemTransactionLoadMetadata::Stellar {
                sequence,
                is_destination_address_exist,
            } => TransactionLoadMetadata::Stellar {
                sequence,
                is_destination_address_exist,
            },
            GemTransactionLoadMetadata::Xrp { sequence, block_number } => TransactionLoadMetadata::Xrp { sequence, block_number },
            GemTransactionLoadMetadata::Algorand {
                sequence,
                block_hash,
                chain_id,
            } => TransactionLoadMetadata::Algorand {
                sequence,
                block_hash,
                chain_id,
            },
            GemTransactionLoadMetadata::Aptos { sequence, data } => TransactionLoadMetadata::Aptos { sequence, data },
            GemTransactionLoadMetadata::Polkadot {
                sequence,
                genesis_hash,
                block_hash,
                block_number,
                spec_version,
                transaction_version,
                period,
            } => TransactionLoadMetadata::Polkadot {
                sequence,
                genesis_hash,
                block_hash,
                block_number,
                spec_version,
                transaction_version,
                period,
            },
            GemTransactionLoadMetadata::Tron {
                block_number,
                block_version,
                block_timestamp,
                transaction_tree_root,
                parent_hash,
                witness_address,
                votes,
            } => TransactionLoadMetadata::Tron {
                block_number,
                block_version,
                block_timestamp,
                transaction_tree_root,
                parent_hash,
                witness_address,
                votes,
            },
            GemTransactionLoadMetadata::Sui { message_bytes } => TransactionLoadMetadata::Sui { message_bytes },
            GemTransactionLoadMetadata::Hyperliquid { order } => TransactionLoadMetadata::Hyperliquid { order },
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSuiCoin {
    pub coin_type: String,
    pub balance: String,
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

impl From<TransactionInputType> for GemTransactionInputType {
    fn from(value: TransactionInputType) -> Self {
        match value {
            TransactionInputType::Transfer(asset) => GemTransactionInputType::Transfer { asset },
            TransactionInputType::Deposit(asset) => GemTransactionInputType::Deposit { asset },
            TransactionInputType::Swap(from_asset, to_asset, swap_data) => GemTransactionInputType::Swap {
                from_asset,
                to_asset,
                swap_data,
            },
            TransactionInputType::Stake(asset, stake_type) => GemTransactionInputType::Stake {
                asset,
                stake_type: stake_type.into(),
            },
            TransactionInputType::TokenApprove(asset, approval_data) => GemTransactionInputType::TokenApprove { asset, approval_data },
            TransactionInputType::Generic(asset, metadata, extra) => GemTransactionInputType::Generic {
                asset,
                metadata,
                extra: extra.into(),
            },
            TransactionInputType::TransferNft(asset, nft_asset) => GemTransactionInputType::TransferNft { asset, nft_asset },
            TransactionInputType::Account(asset, account_type) => GemTransactionInputType::Account { asset, account_type },
            TransactionInputType::Perpetual(asset, perpetual_type) => GemTransactionInputType::Perpetual {
                asset,
                perpetual_type,
            },
        }
    }
}

impl From<GemStakeType> for StakeType {
    fn from(value: GemStakeType) -> Self {
        match value {
            GemStakeType::Delegate { validator } => StakeType::Stake(validator),
            GemStakeType::Undelegate { delegation } => StakeType::Unstake(delegation),
            GemStakeType::Redelegate { delegation, to_validator } => StakeType::Redelegate(primitives::RedelegateData { delegation, to_validator }),
            GemStakeType::WithdrawRewards { validators } => StakeType::Rewards(validators.into_iter().collect()),
            GemStakeType::Withdraw { delegation } => StakeType::Withdraw(delegation),
            GemStakeType::Freeze { freeze_data } => StakeType::Freeze(freeze_data.into()),
        }
    }
}

impl From<StakeType> for GemStakeType {
    fn from(value: StakeType) -> Self {
        match value {
            StakeType::Stake(validator) => GemStakeType::Delegate { validator },
            StakeType::Unstake(delegation) => GemStakeType::Undelegate { delegation },
            StakeType::Redelegate(data) => GemStakeType::Redelegate {
                delegation: data.delegation,
                to_validator: data.to_validator,
            },
            StakeType::Rewards(validators) => GemStakeType::WithdrawRewards { validators },
            StakeType::Withdraw(delegation) => GemStakeType::Withdraw { delegation },
            StakeType::Freeze(freeze_data) => GemStakeType::Freeze {
                freeze_data: freeze_data.into(),
            },
        }
    }
}

impl From<GemTransferDataExtra> for TransferDataExtra {
    fn from(value: GemTransferDataExtra) -> Self {
        TransferDataExtra {
            to: value.to,
            gas_limit: value.gas_limit.map(|s| s.parse().unwrap_or_default()),
            gas_price: value.gas_price.map(|gp| gp.into()),
            data: value.data,
            output_type: value.output_type,
            output_action: value.output_action,
        }
    }
}

impl From<TransferDataExtra> for GemTransferDataExtra {
    fn from(value: TransferDataExtra) -> Self {
        GemTransferDataExtra {
            to: value.to,
            gas_limit: value.gas_limit.map(|gl| gl.to_string()),
            gas_price: value.gas_price.map(|gp| gp.into()),
            data: value.data,
            output_type: value.output_type,
            output_action: value.output_action,
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
            options: options.into_iter().map(|(key, value)| (key, value.to_string())).collect(),
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
                .map(|(key, value)| (key, value.parse().unwrap_or_default()))
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

impl From<GemTransactionInputType> for TransactionInputType {
    fn from(value: GemTransactionInputType) -> Self {
        match value {
            GemTransactionInputType::Transfer { asset } => TransactionInputType::Transfer(asset),
            GemTransactionInputType::Deposit { asset } => TransactionInputType::Deposit(asset),
            GemTransactionInputType::Swap {
                from_asset,
                to_asset,
                swap_data,
            } => TransactionInputType::Swap(
                from_asset,
                to_asset,
                GemSwapData {
                    quote: swap_data.quote,
                    data: swap_data.data,
                },
            ),
            GemTransactionInputType::Stake { asset, stake_type } => TransactionInputType::Stake(asset, stake_type.into()),
            GemTransactionInputType::TokenApprove { asset, approval_data } => TransactionInputType::TokenApprove(
                asset,
                GemApprovalData {
                    token: approval_data.token,
                    spender: approval_data.spender,
                    value: approval_data.value,
                },
            ),
            GemTransactionInputType::Generic { asset, metadata, extra } => TransactionInputType::Generic(asset, metadata, extra.into()),
            GemTransactionInputType::TransferNft { asset, nft_asset } => TransactionInputType::TransferNft(asset, nft_asset),
            GemTransactionInputType::Account { asset, account_type } => TransactionInputType::Account(asset, account_type),
            GemTransactionInputType::Perpetual { asset, perpetual_type } => TransactionInputType::Perpetual(asset, perpetual_type),
        }
    }
}

impl From<GemFreezeData> for FreezeData {
    fn from(value: GemFreezeData) -> Self {
        FreezeData {
            freeze_type: value.freeze_type,
            resource: value.resource,
        }
    }
}

impl From<FreezeData> for GemFreezeData {
    fn from(value: FreezeData) -> Self {
        GemFreezeData {
            freeze_type: value.freeze_type,
            resource: value.resource,
        }
    }
}
