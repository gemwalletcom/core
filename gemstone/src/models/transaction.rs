use crate::models::*;
use num_bigint::BigInt;
use primitives::stake_type::{FreezeData, StakeData};
use primitives::{
    FeeOption, GasPriceType, PerpetualDirection, StakeType, TransactionChange, TransactionFee, TransactionInputType, TransactionLoadInput,
    TransactionLoadMetadata, TransactionMetadata, TransactionPerpetualMetadata, TransactionStateRequest, TransactionUpdate, TransferDataExtra,
    TransferDataOutputType, WalletConnectionSessionAppMetadata,
};
use std::collections::HashMap;

pub type GemPerpetualDirection = PerpetualDirection;
pub type GemFeeOption = FeeOption;
pub type GemTransferDataOutputType = TransferDataOutputType;

#[uniffi::remote(Enum)]
pub enum PerpetualDirection {
    Short,
    Long,
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemStakeData {
    pub data: Option<String>,
    pub to: Option<String>,
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletConnectionSessionAppMetadata {
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon: String,
    pub redirect_native: Option<String>,
    pub redirect_universal: Option<String>,
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapData {
    pub quote: GemSwapQuote,
    pub data: GemSwapQuoteData,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapQuote {
    pub from_value: String,
    pub to_value: String,
    pub provider_data: GemSwapProviderData,
    pub wallet_address: String,
    pub slippage_bps: u32,
    pub eta_in_seconds: Option<u32>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapQuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
    pub approval: Option<GemApprovalData>,
    pub gas_limit: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapProviderData {
    pub provider: String,
    pub name: String,
    pub protocol_name: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPerpetualConfirmData {
    pub direction: GemPerpetualDirection,
    pub asset: GemAsset,
    pub asset_index: i32,
    pub price: String,
    pub fiat_value: f64,
    pub size: String,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemPerpetualType {
    Open { data: GemPerpetualConfirmData },
    Close { data: GemPerpetualConfirmData },
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
    Perpetual {
        asset: GemAsset,
        perpetual_type: GemPerpetualType,
    },
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
        approve_agent_required: bool,
        approve_referral_required: bool,
        approve_builder_required: bool,
        builder_fee_bps: u32,
        agent_address: String,
        agent_private_key: String,
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
                token_program: token_program.map(|tp| tp.into()),
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
            TransactionLoadMetadata::Bitcoin { utxos } => GemTransactionLoadMetadata::Bitcoin {
                utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
            },
            TransactionLoadMetadata::Cardano { utxos } => GemTransactionLoadMetadata::Cardano {
                utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
            },
            TransactionLoadMetadata::Evm { nonce, chain_id, stake_data } => GemTransactionLoadMetadata::Evm {
                nonce,
                chain_id,
                stake_data: stake_data.map(|sd| sd.into()),
            },
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
            TransactionLoadMetadata::Aptos { sequence } => GemTransactionLoadMetadata::Aptos { sequence },
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
            TransactionLoadMetadata::Hyperliquid {
                approve_agent_required,
                approve_referral_required,
                approve_builder_required,
                builder_fee_bps,
                agent_address,
                agent_private_key,
            } => GemTransactionLoadMetadata::Hyperliquid {
                approve_agent_required,
                approve_referral_required,
                approve_builder_required,
                builder_fee_bps,
                agent_address,
                agent_private_key,
            },
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
                token_program: token_program.map(|tp| tp.into()),
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
            GemTransactionLoadMetadata::Bitcoin { utxos } => TransactionLoadMetadata::Bitcoin {
                utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
            },
            GemTransactionLoadMetadata::Cardano { utxos } => TransactionLoadMetadata::Cardano {
                utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
            },
            GemTransactionLoadMetadata::Evm { nonce, chain_id, stake_data } => TransactionLoadMetadata::Evm {
                nonce,
                chain_id,
                stake_data: stake_data.map(|sd| sd.into()),
            },
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
            GemTransactionLoadMetadata::Aptos { sequence } => TransactionLoadMetadata::Aptos { sequence },
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
            GemTransactionLoadMetadata::Hyperliquid {
                approve_agent_required,
                approve_referral_required,
                approve_builder_required,
                builder_fee_bps,
                agent_address,
                agent_private_key,
            } => TransactionLoadMetadata::Hyperliquid {
                approve_agent_required,
                approve_referral_required,
                approve_builder_required,
                builder_fee_bps,
                agent_address,
                agent_private_key,
            },
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSuiCoin {
    pub coin_type: String,
    pub balance: String,
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
            GemStakeType::Delegate { validator } => StakeType::Stake(validator.into()),
            GemStakeType::Undelegate { delegation } => StakeType::Unstake(delegation.into()),
            GemStakeType::Redelegate { delegation, to_validator } => StakeType::Redelegate(primitives::RedelegateData {
                delegation: delegation.into(),
                to_validator: to_validator.into(),
            }),
            GemStakeType::WithdrawRewards { validators } => StakeType::Rewards(validators.into_iter().map(|v| v.into()).collect()),
            GemStakeType::Withdraw { delegation } => StakeType::Withdraw(delegation.into()),
            GemStakeType::Freeze { freeze_data } => StakeType::Freeze(freeze_data.into()),
        }
    }
}

impl From<StakeType> for GemStakeType {
    fn from(value: StakeType) -> Self {
        match value {
            StakeType::Stake(validator) => GemStakeType::Delegate { validator: validator.into() },
            StakeType::Unstake(delegation) => GemStakeType::Undelegate { delegation: delegation.into() },
            StakeType::Redelegate(data) => GemStakeType::Redelegate {
                delegation: data.delegation.into(),
                to_validator: data.to_validator.into(),
            },
            StakeType::Rewards(validators) => GemStakeType::WithdrawRewards {
                validators: validators.into_iter().map(|v| v.into()).collect(),
            },
            StakeType::Withdraw(delegation) => GemStakeType::Withdraw { delegation: delegation.into() },
            StakeType::Freeze(freeze_data) => GemStakeType::Freeze {
                freeze_data: freeze_data.into(),
            },
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

impl From<GemStakeData> for StakeData {
    fn from(value: GemStakeData) -> Self {
        StakeData {
            data: value.data,
            to: value.to,
        }
    }
}

impl From<StakeData> for GemStakeData {
    fn from(value: StakeData) -> Self {
        GemStakeData {
            data: value.data,
            to: value.to,
        }
    }
}

impl From<WalletConnectionSessionAppMetadata> for GemWalletConnectionSessionAppMetadata {
    fn from(value: WalletConnectionSessionAppMetadata) -> Self {
        GemWalletConnectionSessionAppMetadata {
            name: value.name,
            description: value.description,
            url: value.url,
            icon: value.icon,
            redirect_native: value.redirect_native,
            redirect_universal: value.redirect_universal,
        }
    }
}

impl From<GemTransferDataExtra> for TransferDataExtra {
    fn from(value: GemTransferDataExtra) -> Self {
        TransferDataExtra {
            gas_limit: value.gas_limit.map(|s| s.parse().unwrap_or_default()),
            gas_price: value.gas_price.map(|gp| gp.into()),
            data: value.data,
            output_type: value.output_type,
        }
    }
}

impl From<TransferDataExtra> for GemTransferDataExtra {
    fn from(value: TransferDataExtra) -> Self {
        GemTransferDataExtra {
            gas_limit: value.gas_limit.map(|gl| gl.to_string()),
            gas_price: value.gas_price.map(|gp| gp.into()),
            data: value.data,
            output_type: value.output_type,
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
        use primitives::{swap::ApprovalData, swap::SwapData};
        match value {
            GemTransactionInputType::Transfer { asset } => TransactionInputType::Transfer(asset.into()),
            GemTransactionInputType::Deposit { asset } => TransactionInputType::Deposit(asset.into()),
            GemTransactionInputType::Swap {
                from_asset,
                to_asset,
                swap_data,
            } => TransactionInputType::Swap(
                from_asset.into(),
                to_asset.into(),
                SwapData {
                    quote: swap_data.quote.into(),
                    data: swap_data.data.into(),
                },
            ),
            GemTransactionInputType::Stake { asset, stake_type } => TransactionInputType::Stake(asset.into(), stake_type.into()),
            GemTransactionInputType::TokenApprove { asset, approval_data } => TransactionInputType::TokenApprove(
                asset.into(),
                ApprovalData {
                    token: approval_data.token,
                    spender: approval_data.spender,
                    value: approval_data.value.parse().unwrap_or_default(),
                },
            ),
            GemTransactionInputType::Generic { asset, metadata, extra } => TransactionInputType::Generic(asset.into(), metadata.into(), extra.into()),
            GemTransactionInputType::Perpetual { asset, perpetual_type } => TransactionInputType::Perpetual(asset.into(), perpetual_type.into()),
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

impl From<GemSwapQuote> for primitives::swap::SwapQuote {
    fn from(value: GemSwapQuote) -> Self {
        primitives::swap::SwapQuote {
            from_value: value.from_value.parse().unwrap_or_default(),
            to_value: value.to_value.parse().unwrap_or_default(),
            provider_data: value.provider_data.into(),
            wallet_address: value.wallet_address,
            slippage_bps: value.slippage_bps,
            eta_in_seconds: value.eta_in_seconds,
        }
    }
}

impl From<GemSwapProviderData> for primitives::swap::SwapProviderData {
    fn from(value: GemSwapProviderData) -> Self {
        primitives::swap::SwapProviderData {
            provider: primitives::SwapProvider::UniswapV3,
            name: value.name,
            protocol_name: value.protocol_name,
        }
    }
}

impl From<GemSwapQuoteData> for primitives::swap::SwapQuoteData {
    fn from(value: GemSwapQuoteData) -> Self {
        primitives::swap::SwapQuoteData {
            to: value.to,
            value: value.value.parse().unwrap_or_default(),
            data: value.data,
            approval: value.approval.map(|a| primitives::swap::ApprovalData {
                token: a.token,
                spender: a.spender,
                value: a.value.parse().unwrap_or_default(),
            }),
            gas_limit: value.gas_limit.map(|gl| gl.parse().unwrap_or_default()),
        }
    }
}
