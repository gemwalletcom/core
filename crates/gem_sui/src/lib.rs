#[cfg(feature = "rpc")]
pub mod rpc;

pub mod jsonrpc;
pub mod model;
pub mod object_id;
pub mod stake;
pub mod transfer;
pub mod tx;

use anyhow::{anyhow, Error};
use model::Coin;
pub use object_id::ObjectID;
pub use stake::*;
use sui_transaction_builder::unresolved::Input;
pub use transfer::*;

pub static SUI_SYSTEM_ID: &str = "sui_system";

pub static SUI_FRAMEWORK_PACKAGE_ID: u8 = 0x2;
pub static SUI_SYSTEM_PACKAGE_ID: u8 = 0x3;
pub static SUI_SYSTEM_STATE_OBJECT_ID: u8 = 0x5;
pub static SUI_CLOCK_OBJECT_ID: u8 = 0x6;

pub const SUI_COIN_TYPE: &str = "0x2::sui::SUI";
pub static SUI_COIN_TYPE_FULL: &str = "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI";
pub static EMPTY_ADDRESS: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
pub static STORAGE_FEE_UNIT: u64 = 76; // https://blog.sui.io/storage-fees-explained

pub fn sui_system_state_object_input() -> Input {
    Input::shared(ObjectID::from(SUI_SYSTEM_STATE_OBJECT_ID).id(), 1, true)
}

pub fn sui_clock_object_input() -> Input {
    Input::shared(ObjectID::from(SUI_CLOCK_OBJECT_ID).id(), 1, false)
}

pub fn validate_enough_balance(coins: &[Coin], amount: u64) -> Option<Error> {
    if coins.is_empty() {
        return Some(anyhow!("coins list is empty"));
    }

    let total_amount: u64 = coins.iter().map(|x| x.balance).sum();
    if total_amount < amount {
        return Some(anyhow!(format!("total amount ({}) is less than amount to send ({})", total_amount, amount),));
    }
    None
}
