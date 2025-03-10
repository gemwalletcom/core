use lazy_static::lazy_static;
use num_bigint::BigInt;

// String constants for BigInt parsing
pub const MAX_SQRT_PRICE_STR: &str = "79226673515401279992447579055";
pub const MIN_SQRT_PRICE_STR: &str = "4295048016";

// Log calculation constants
pub const LOG_B_2_X32: &str = "59543866431248";
pub const LOG_B_P_ERR_MARGIN_LOWER_X64: &str = "184467440737095516";
pub const LOG_B_P_ERR_MARGIN_UPPER_X64: &str = "15793534762490258745";

// Lazy initialized BigInt constants
lazy_static! {
    // Common BigInt values
    pub static ref ZERO: BigInt = BigInt::from(0);
    pub static ref ONE: BigInt = BigInt::from(1);
    pub static ref TWO: BigInt = BigInt::from(2);

    // Maximum values
    pub static ref U64_MAX: BigInt = BigInt::from(u64::MAX);
    pub static ref U128: BigInt = BigInt::from(2).pow(128);
    pub static ref U128_MAX: BigInt = BigInt::from(2).pow(128) - BigInt::from(1);

    // Sqrt price limits
    pub static ref MAX_SQRT_PRICE: BigInt = BigInt::parse_bytes(MAX_SQRT_PRICE_STR.as_bytes(), 10).unwrap();
    pub static ref MIN_SQRT_PRICE: BigInt = BigInt::parse_bytes(MIN_SQRT_PRICE_STR.as_bytes(), 10).unwrap();

    // Log calculation values
    pub static ref LOG_B_2_X32_VALUE: BigInt = BigInt::parse_bytes(LOG_B_2_X32.as_bytes(), 10).unwrap();
    pub static ref LOG_B_P_ERR_MARGIN_LOWER_X64_VALUE: BigInt = BigInt::parse_bytes(LOG_B_P_ERR_MARGIN_LOWER_X64.as_bytes(), 10).unwrap();
    pub static ref LOG_B_P_ERR_MARGIN_UPPER_X64_VALUE: BigInt = BigInt::parse_bytes(LOG_B_P_ERR_MARGIN_UPPER_X64.as_bytes(), 10).unwrap();

    pub static ref FEE_RATE_DENOMINATOR: BigInt = BigInt::from(1000000);
}
