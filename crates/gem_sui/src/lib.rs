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

pub mod gas_budget;
pub mod jsonrpc;
pub mod operations;

#[cfg(feature = "signer")]
pub mod signer;

use models::Coin;
pub use models::ObjectId;
pub use operations::*;
use std::error::Error;
use sui_transaction_builder::unresolved::Input;

pub const SUI_SYSTEM_ID: &str = "sui_system";

pub const SUI_FRAMEWORK_PACKAGE_ID: u8 = 0x2;
pub const SUI_SYSTEM_PACKAGE_ID: u8 = 0x3;
pub const SUI_SYSTEM_STATE_OBJECT_ID: u8 = 0x5;
pub const SUI_CLOCK_OBJECT_ID: u8 = 0x6;

pub const SUI_COIN_TYPE: &str = "0x2::sui::SUI";
pub const SUI_COIN_TYPE_FULL: &str = "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI";
pub const EMPTY_ADDRESS: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
pub const STORAGE_FEE_UNIT: u64 = 76; // https://blog.sui.io/storage-fees-explained
pub const SUI_STAKE_EVENT: &str = "0x3::validator::StakingRequestEvent";
pub const SUI_UNSTAKE_EVENT: &str = "0x3::validator::UnstakingRequestEvent";

pub fn sui_system_state_object_input() -> Input {
    Input::shared(ObjectId::from(SUI_SYSTEM_STATE_OBJECT_ID).into(), 1, true)
}

pub fn sui_clock_object_input() -> Input {
    Input::shared(ObjectId::from(SUI_CLOCK_OBJECT_ID).into(), 1, false)
}

pub fn validate_enough_balance(coins: &[Coin], amount: u64) -> Option<Box<dyn Error + Send + Sync>> {
    if coins.is_empty() {
        return Some("coins list is empty".into());
    }

    let total_amount: u64 = coins.iter().map(|x| x.balance).sum();
    if total_amount < amount {
        return Some(format!("total amount ({}) is less than amount to send ({})", total_amount, amount).into());
    }
    None
}
