pub mod address;
pub use address::validate_address;
pub mod coin_type;
pub use coin_type::{coin_type_matches, full_coin_type, is_sui_coin};
#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "rpc")]
pub use rpc::SuiClient;

#[cfg(feature = "rpc")]
pub mod provider;

pub mod models;

#[cfg(feature = "rpc")]
pub mod transfer_builder;
#[cfg(feature = "rpc")]
pub use transfer_builder::*;

pub mod error;
pub mod gas_budget;
pub mod tx_builder;

#[cfg(feature = "signer")]
pub mod signer;

pub use error::SuiError;
pub use models::ObjectId;
use models::{Coin, OwnedCoins};
use std::error::Error;
use sui_transaction_builder::ObjectInput;
use sui_types::Address;
pub use tx_builder::{decode_transaction, stake::*, transfer::*, validate_and_hash};

pub const SUI_SYSTEM_ID: &str = "sui_system";

pub const SUI_FRAMEWORK_PACKAGE_ID: u8 = 0x2;
pub const SUI_SYSTEM_PACKAGE_ID: u8 = 0x3;
pub const SUI_SYSTEM_STATE_OBJECT_ID: u8 = 0x5;
pub const SUI_CLOCK_OBJECT_ID: u8 = 0x6;

pub const SUI_COIN_TYPE: &str = "0x2::sui::SUI";
pub const SUI_COIN_TYPE_FULL: &str = "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI";
pub const EMPTY_ADDRESS: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
pub const STORAGE_FEE_UNIT: u64 = 76; // https://blog.sui.io/storage-fees-explained
pub const ESTIMATION_GAS_BUDGET: u64 = 50_000_000;
pub const SUI_STAKE_EVENT: &str = "0x3::validator::StakingRequestEvent";
pub const SUI_UNSTAKE_EVENT: &str = "0x3::validator::UnstakingRequestEvent";

pub fn sui_framework_package_address() -> Address {
    ObjectId::from(SUI_FRAMEWORK_PACKAGE_ID).into()
}

pub fn sui_system_package_address() -> Address {
    ObjectId::from(SUI_SYSTEM_PACKAGE_ID).into()
}

pub fn sui_system_state_object_id() -> Address {
    ObjectId::from(SUI_SYSTEM_STATE_OBJECT_ID).into()
}

pub fn sui_clock_object_id() -> Address {
    ObjectId::from(SUI_CLOCK_OBJECT_ID).into()
}

pub fn sui_system_state_object_input() -> ObjectInput {
    ObjectInput::shared(sui_system_state_object_id(), 1, true)
}

pub fn sui_clock_object_input() -> ObjectInput {
    ObjectInput::shared(sui_clock_object_id(), 1, false)
}

pub fn validate_enough_balance(coins: &OwnedCoins<Coin>, amount: u64) -> Option<Box<dyn Error + Send + Sync>> {
    let total = coins.total();
    if total == 0 {
        return Some("no spendable coin objects or address balance".into());
    }
    if total < amount {
        return Some(format!("total amount ({}) is less than amount to send ({})", total, amount).into());
    }
    None
}
