use primitives::stake_type::{FreezeType, Resource};
use primitives::StakeChain;

pub type GemFreezeType = FreezeType;
pub type GemResource = Resource;
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
    Monad,
    Tron,
    Aptos,
    HyperCore,
}
