use crate::models::custom_types::{DateTimeUtc, GemBigUint};
use primitives::{AssetId, Chain, EarnPositionData, EarnPosition, EarnPositionState, EarnProvider, EarnProviderType, Price, YieldType};

pub type GemPrice = Price;

#[uniffi::remote(Record)]
pub struct GemPrice {
    pub price: f64,
    pub price_change_percentage_24h: f64,
    pub updated_at: DateTimeUtc,
}

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
    pub commission: f64,
    pub apr: f64,
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

pub type GemEarnPositionData = EarnPositionData;

#[uniffi::remote(Record)]
pub struct GemEarnPositionData {
    pub asset_id: AssetId,
    pub state: GemEarnPositionState,
    pub balance: GemBigUint,
    pub shares: GemBigUint,
    pub rewards: GemBigUint,
    pub completion_date: Option<DateTimeUtc>,
    pub position_id: String,
    pub provider_id: String,
}

pub type GemEarnPosition = EarnPosition;

#[uniffi::remote(Record)]
pub struct GemEarnPosition {
    pub data: GemEarnPositionData,
    pub provider: GemEarnProvider,
    pub price: Option<GemPrice>,
}

pub type GemYieldType = YieldType;

#[uniffi::remote(Enum)]
pub enum GemYieldType {
    Deposit,
    Withdraw,
}
