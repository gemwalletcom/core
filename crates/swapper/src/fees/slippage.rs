use alloy_primitives::U256;
use number_formatter::{BigNumberFormatter, NumberFormatterError};
use std::ops::{Div, Mul};

const HUNDRED_PERCENT_IN_BPS: u32 = 10000;
const BPS_PER_PERCENT_DECIMALS: i32 = 2;

pub trait BasisPointConvert: Sized + Copy {
    fn from_u32(value: u32) -> Self;
}

impl BasisPointConvert for U256 {
    fn from_u32(value: u32) -> Self {
        Self::from(value)
    }
}

impl BasisPointConvert for u128 {
    fn from_u32(value: u32) -> Self {
        value as u128
    }
}

impl BasisPointConvert for u64 {
    fn from_u32(value: u32) -> Self {
        value as u64
    }
}

pub fn apply_slippage_in_bp<T>(amount: &T, bps: u32) -> T
where
    T: BasisPointConvert + Mul<Output = T> + Div<Output = T>,
{
    let basis_points = T::from_u32(HUNDRED_PERCENT_IN_BPS);
    let slippage = T::from_u32(HUNDRED_PERCENT_IN_BPS - bps.min(HUNDRED_PERCENT_IN_BPS));
    (*amount * slippage) / basis_points
}

pub fn bps_to_percent_string(bps: u32) -> Result<String, NumberFormatterError> {
    BigNumberFormatter::value(&bps.to_string(), BPS_PER_PERCENT_DECIMALS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_slippage_in_bp() {
        assert_eq!(apply_slippage_in_bp(&U256::from(100), 300), U256::from(97));
        assert_eq!(apply_slippage_in_bp(&100_u128, 300), 97_u128);
        assert_eq!(apply_slippage_in_bp(&1000_u64, 500), 950_u64);
        assert_eq!(apply_slippage_in_bp(&U256::from(1000), 0), U256::from(1000));
        assert_eq!(apply_slippage_in_bp(&U256::from(1000), HUNDRED_PERCENT_IN_BPS), U256::ZERO);
    }

    #[test]
    fn test_bps_to_percent_string() {
        assert_eq!(bps_to_percent_string(100).unwrap(), "1");
        assert_eq!(bps_to_percent_string(50).unwrap(), "0.5");
        assert_eq!(bps_to_percent_string(200).unwrap(), "2");
        assert_eq!(bps_to_percent_string(10).unwrap(), "0.1");
        assert_eq!(bps_to_percent_string(0).unwrap(), "0");
    }
}
