use primitives::stake_type::{FreezeType, Resource};

pub type GemFreezeType = FreezeType;
pub type GemResource = Resource;

#[uniffi::remote(Enum)]
pub enum FreezeType {
    Freeze,
    Unfreeze,
}

#[uniffi::remote(Enum)]
pub enum Resource {
    Bandwidth,
    Energy,
}