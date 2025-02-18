use alloy_core::primitives::U256;
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

pub fn apply_slippage_in_bp<T>(amount: &T, bps: u32) -> T
where
    T: BasisPointConvert + Mul<Output = T> + Div<Output = T>,
{
    let basis_points = T::from_u32(HUNDRED_PERCENT_IN_BPS);
    let slippage = T::from_u32(HUNDRED_PERCENT_IN_BPS - bps);
    (*amount * slippage) / basis_points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_slippage_u256() {
        let amount = U256::from(100);
        let result = apply_slippage_in_bp(&amount, 300);

        assert_eq!(result, U256::from(97));
    }

    #[test]
    fn test_apply_slippage_u128() {
        let amount = 100_u128;
        let result = apply_slippage_in_bp(&amount, 300);

        assert_eq!(result, 97_u128);
    }
}
