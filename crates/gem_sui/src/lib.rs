pub mod decoder;
pub mod jsonrpc;
pub mod model;
pub mod stake;
pub mod transfer;
pub use decoder::*;
pub use stake::*;
pub use transfer::*;

use anyhow::{anyhow, Error};
use model::Coin;
use sui_types::{Address, ObjectId};

static SUI_SYSTEM_ID: &str = "sui_system";
static SUI_REQUEST_ADD_STAKE: &str = "request_add_stake";
static SUI_REQUEST_WITHDRAW_STAKE: &str = "request_withdraw_stake";
static SUI_SYSTEM_ADDRESS: u8 = 0x3;

pub static SUI_SYSTEM_STATE_OBJECT_ID: u8 = 0x5;
pub static SUI_CLOCK_OBJECT_ID: u8 = 0x6;
pub static SUI_FRAMEWORK_PACKAGE_ID: u8 = 0x2;

pub static SUI_COIN_TYPE: &str = "0x2::sui::SUI";
pub static SUI_COIN_TYPE_FULL: &str = "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI";
pub static EMPTY_ADDRESS: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
pub static STORAGE_FEE_UNIT: u64 = 76; // https://blog.sui.io/storage-fees-explained

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

pub fn address_from_u8(byte: u8) -> Address {
    let mut bytes = [0u8; Address::LENGTH];
    bytes[31] = byte;
    Address::new(bytes)
}

pub fn object_id_from_u8(byte: u8) -> ObjectId {
    let mut bytes = [0u8; ObjectId::LENGTH];
    bytes[31] = byte;
    ObjectId::new(bytes)
}
