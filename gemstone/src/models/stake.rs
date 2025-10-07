use crate::models::custom_types::{DateTimeUtc, GemBigUint};
use primitives::stake_type::{FreezeType, Resource};
use primitives::{AssetId, Chain, Delegation, DelegationBase, DelegationState, DelegationValidator, Price, StakeChain};

pub type GemFreezeType = FreezeType;
pub type GemResource = Resource;
pub type GemDelegation = Delegation;
pub type GemDelegationBase = DelegationBase;
pub type GemDelegationValidator = DelegationValidator;
pub type GemDelegationState = DelegationState;
pub type GemPrice = Price;
pub type GemStakeChain = StakeChain;

#[uniffi::remote(Enum)]
pub enum GemFreezeType {
    Freeze,
    Unfreeze,
}

#[uniffi::remote(Enum)]
pub enum GemResource {
    Bandwidth,
    Energy,
}

#[uniffi::remote(Enum)]
pub enum GemStakeChain {
    Cosmos,
    Osmosis,
    Injective,
    Sei,
    Celestia,
    Ethereum,
    Solana,
    Sui,
    SmartChain,
    Tron,
    Aptos,
    HyperCore,
}

#[uniffi::remote(Enum)]
pub enum GemDelegationState {
    Active,
    Pending,
    Undelegating,
    Inactive,
    Activating,
    Deactivating,
    AwaitingWithdrawal,
}

#[uniffi::remote(Record)]
pub struct GemDelegationValidator {
    pub chain: Chain,
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub commission: f64,
    pub apr: f64,
}

#[uniffi::remote(Record)]
pub struct GemDelegationBase {
    pub asset_id: AssetId,
    pub state: GemDelegationState,
    pub balance: GemBigUint,
    pub shares: GemBigUint,
    pub rewards: GemBigUint,
    pub completion_date: Option<DateTimeUtc>,
    pub delegation_id: String,
    pub validator_id: String,
}

#[uniffi::remote(Record)]
pub struct GemPrice {
    pub price: f64,
    pub price_change_percentage_24h: f64,
    pub updated_at: DateTimeUtc,
}

#[uniffi::remote(Record)]
pub struct GemDelegation {
    pub base: GemDelegationBase,
    pub validator: GemDelegationValidator,
    pub price: Option<GemPrice>,
}
