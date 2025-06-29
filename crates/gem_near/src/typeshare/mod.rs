pub mod near_account;
pub mod near_block;
pub mod near_error;
pub mod near_fee;
pub mod near_transaction;

pub use near_account::*;
pub use near_block::*;
pub use near_error::*;
pub use near_fee::*;
pub use near_transaction::*;

type Int = i64;
type UInt64 = u64;
