use num_bigint::BigInt;

pub mod constants;
pub mod error;
pub mod math;
pub mod swap;
pub mod tick;

pub use swap::compute_swap;

#[derive(Clone, Default)]
pub struct ClmmPoolData {
    pub liquidity: BigInt,
    pub current_tick_index: i32,
    pub current_sqrt_price: BigInt,
    pub fee_rate: BigInt,
}

#[derive(Clone)]
pub struct TickData {
    pub index: i32,
    pub sqrt_price: BigInt,
    pub liquidity_net: BigInt,
}
