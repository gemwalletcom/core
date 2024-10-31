use alloy_core::primitives::U256;

pub fn apply_slippage_in_bp(amount: &U256, bps: u32) -> U256 {
    amount * U256::from(10000 - bps) / U256::from(10000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_core::primitives::U256;

    #[test]
    fn test_apply_slippage() {
        let amount = U256::from(100);
        let expected = U256::from(97);

        let result = apply_slippage_in_bp(&amount, 300);

        assert_eq!(result, expected);
    }
}
