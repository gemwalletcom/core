use alloy_primitives::U256;
use std::ops::{Div, Mul};

const HUNDRED_PERCENT_IN_BPS: u32 = 10000;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_slippage() {
        assert_eq!(apply_slippage_in_bp(&U256::from(100), 300), U256::from(97));
        assert_eq!(apply_slippage_in_bp(&100_u128, 300), 97_u128);
        assert_eq!(apply_slippage_in_bp(&1000_u64, 500), 950_u64);
        assert_eq!(apply_slippage_in_bp(&U256::from(1000), 0), U256::from(1000));
        assert_eq!(apply_slippage_in_bp(&U256::from(1000), HUNDRED_PERCENT_IN_BPS), U256::ZERO);
        assert_eq!(apply_slippage_in_bp(&U256::from(1000), 12000), U256::ZERO);
        assert_eq!(apply_slippage_in_bp(&U256::from(1000), 300 + 9800), U256::ZERO);
    }
}
