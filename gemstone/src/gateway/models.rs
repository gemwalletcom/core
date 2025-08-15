use primitives::{AssetBalance, AssetId, Balance, Chain, FeePriorityValue, UTXO};

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemAssetBalance {
    pub asset_id: AssetId,
    pub balance: GemBalance,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemBalance {
    pub available: String,
    pub frozen: String,
    pub locked: String,
    pub staked: String,
    pub pending: String,
    pub rewards: String,
    pub reserved: String,
    pub withdrawable: String,
}

impl From<AssetBalance> for GemAssetBalance {
    fn from(value: AssetBalance) -> Self {
        Self {
            asset_id: value.asset_id,
            balance: value.balance.into(),
        }
    }
}

impl From<Balance> for GemBalance {
    fn from(value: Balance) -> Self {
        Self {
            available: value.available,
            frozen: value.frozen,
            locked: value.locked,
            staked: value.staked,
            pending: value.pending,
            rewards: value.rewards,
            reserved: value.reserved,
            withdrawable: value.withdrawable,
        }
    }
}

impl GemBalance {
    pub fn coin_balance(available: String) -> Self {
        Self {
            available,
            frozen: "0".to_string(),
            locked: "0".to_string(),
            staked: "0".to_string(),
            pending: "0".to_string(),
            rewards: "0".to_string(),
            reserved: "0".to_string(),
            withdrawable: "0".to_string(),
        }
    }

    pub fn token_balance(available: String) -> Self {
        Self::coin_balance(available)
    }

    pub fn stake_balance(staked: String, pending: String, rewards: Option<String>) -> Self {
        Self {
            available: "0".to_string(),
            frozen: "0".to_string(),
            locked: "0".to_string(),
            staked,
            pending,
            rewards: rewards.unwrap_or("0".to_string()),
            reserved: "0".to_string(),
            withdrawable: "0".to_string(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDelegationValidator {
    pub chain: Chain,
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub commision: f64,
    pub apr: f64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDelegationBase {
    pub asset_id: AssetId,
    pub state: String,
    pub balance: String,
    pub shares: String,
    pub rewards: String,
    pub completion_date: Option<i64>,
    pub delegation_id: String,
    pub validator_id: String,
}

impl From<primitives::DelegationValidator> for GemDelegationValidator {
    fn from(value: primitives::DelegationValidator) -> Self {
        Self {
            chain: value.chain,
            id: value.id,
            name: value.name,
            is_active: value.is_active,
            commision: value.commision,
            apr: value.apr,
        }
    }
}

impl From<primitives::DelegationBase> for GemDelegationBase {
    fn from(value: primitives::DelegationBase) -> Self {
        Self {
            asset_id: value.asset_id,
            state: value.state.as_ref().to_string(),
            balance: value.balance,
            shares: value.shares,
            rewards: value.rewards,
            completion_date: None,
            delegation_id: value.delegation_id,
            validator_id: value.validator_id,
        }
    }
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
}

impl From<primitives::TransactionChange> for GemTransactionChange {
    fn from(value: primitives::TransactionChange) -> Self {
        match value {
            primitives::TransactionChange::HashChange { old, new } => GemTransactionChange::HashChange { old, new },
            primitives::TransactionChange::Metadata(metadata) => GemTransactionChange::Metadata(metadata.into()),
            primitives::TransactionChange::BlockNumber(block_number) => GemTransactionChange::BlockNumber(block_number),
            primitives::TransactionChange::NetworkFee(fee) => GemTransactionChange::NetworkFee(fee),
        }
    }
}

impl From<primitives::TransactionMetadata> for GemTransactionMetadata {
    fn from(value: primitives::TransactionMetadata) -> Self {
        match value {
            primitives::TransactionMetadata::Perpetual(perp) => GemTransactionMetadata::Perpetual(perp.into()),
        }
    }
}

impl From<primitives::transaction_metadata_types::TransactionPerpetualMetadata> for GemTransactionPerpetualMetadata {
    fn from(value: primitives::transaction_metadata_types::TransactionPerpetualMetadata) -> Self {
        GemTransactionPerpetualMetadata {
            pnl: value.pnl,
            price: value.price,
        }
    }
}

impl From<GemTransactionStateRequest> for primitives::TransactionStateRequest {
    fn from(value: GemTransactionStateRequest) -> Self {
        primitives::TransactionStateRequest {
            id: value.id,
            sender_address: value.sender_address,
            created_at: value.created_at,
        }
    }
}

impl From<primitives::TransactionUpdate> for GemTransactionUpdate {
    fn from(value: primitives::TransactionUpdate) -> Self {
        GemTransactionUpdate {
            state: value.state.to_string(),
            changes: value.changes.into_iter().map(|change| change.into()).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemUTXO {
    pub transaction_id: String,
    pub vout: i32,
    pub value: String,
    pub address: String,
}

impl From<UTXO> for GemUTXO {
    fn from(value: UTXO) -> Self {
        Self {
            transaction_id: value.transaction_id,
            vout: value.vout,
            value: value.value,
            address: value.address,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemFeePriorityValue {
    pub priority: String,
    pub value: String,
}

impl From<FeePriorityValue> for GemFeePriorityValue {
    fn from(value: FeePriorityValue) -> Self {
        Self {
            priority: value.priority.as_ref().to_string(),
            value: value.value,
        }
    }
}
