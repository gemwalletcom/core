pub mod lp;
pub mod relayer;
pub use lp::*;
pub use relayer::*;

use alloy_primitives::U256;
use num_bigint::{BigInt, Sign};

pub fn multiply(amount: U256, percent: BigInt, decimals: u32) -> U256 {
    let amount_big = BigInt::from_bytes_le(Sign::Plus, amount.to_le_bytes::<32>().as_slice());
    let value = amount_big * percent / BigInt::from(10_u64.pow(decimals));
    U256::from_le_slice(&value.to_signed_bytes_le())
}
