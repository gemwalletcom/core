use crate::models::custom_types::{DateTimeUtc, GemBigUint};
use primitives::stake_type::Resource;
use primitives::{AssetId, Chain, Delegation, DelegationBase, DelegationState, DelegationValidator, Price, PriceProvider, StakeChain, StakeProviderType};

pub type GemResource = Resource;
pub type GemDelegation = Delegation;
pub type GemDelegationBase = DelegationBase;
pub type GemDelegationValidator = DelegationValidator;
pub type GemDelegationState = DelegationState;
pub type GemStakeProviderType = StakeProviderType;
pub type GemPrice = Price;
pub type GemPriceProvider = PriceProvider;
pub type GemStakeChain = StakeChain;

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
    Monad,
    Tron,
    Aptos,
    HyperCore,
}

#[uniffi::remote(Enum)]
pub enum GemDelegationState {
    Active,
    Pending,
    Inactive,
    Activating,
    Deactivating,
    AwaitingWithdrawal,
}

#[uniffi::remote(Enum)]
pub enum GemStakeProviderType {
    Stake,
    Earn,
}

#[uniffi::remote(Record)]
pub struct GemDelegationValidator {
    pub chain: Chain,
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub commission: f64,
    pub apr: f64,
    pub provider_type: GemStakeProviderType,
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

#[uniffi::remote(Enum)]
pub enum GemPriceProvider {
    Coingecko,
    Pyth,
    Jupiter,
    DefiLlama,
}

#[uniffi::remote(Record)]
pub struct GemPrice {
    pub price: f64,
    pub price_change_percentage_24h: f64,
    pub updated_at: DateTimeUtc,
    pub provider: PriceProvider,
}

#[uniffi::remote(Record)]
pub struct GemDelegation {
    pub base: GemDelegationBase,
    pub validator: GemDelegationValidator,
    pub price: Option<GemPrice>,
}
