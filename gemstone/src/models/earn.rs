use crate::models::custom_types::{DateTimeUtc, GemBigUint};
use primitives::{AssetId, Chain, EarnPositionBase, EarnPositionState, EarnProvider, EarnProviderType, EarnYieldType};

pub type GemEarnProviderType = EarnProviderType;

#[uniffi::remote(Enum)]
pub enum GemEarnProviderType {
    Stake,
    Yield,
}

pub type GemEarnProvider = EarnProvider;

#[uniffi::remote(Record)]
pub struct GemEarnProvider {
    pub chain: Chain,
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub fee: f64,
    pub apy: f64,
    pub provider_type: GemEarnProviderType,
}

pub type GemEarnPositionState = EarnPositionState;

#[uniffi::remote(Enum)]
pub enum GemEarnPositionState {
    Active,
    Pending,
    Inactive,
    Activating,
    Deactivating,
    AwaitingWithdrawal,
}

pub type GemEarnPositionBase = EarnPositionBase;

#[uniffi::remote(Record)]
pub struct GemEarnPositionBase {
    pub asset_id: AssetId,
    pub state: GemEarnPositionState,
    pub balance: GemBigUint,
    pub shares: GemBigUint,
    pub rewards: GemBigUint,
    pub unlock_date: Option<DateTimeUtc>,
    pub position_id: String,
    pub provider_id: String,
}

pub type GemEarnYieldType = EarnYieldType;

#[uniffi::remote(Enum)]
pub enum GemEarnYieldType {
    Deposit { provider_id: String },
    Withdraw { provider_id: String },
}
