use std::str::FromStr;

use num_bigint::BigUint;
use num_traits::Zero;

use crate::SwapperError;

pub fn scale_units(value: BigUint, from_decimals: u32, to_decimals: u32) -> Result<BigUint, SwapperError> {
    if from_decimals == to_decimals {
        return Ok(value);
    }

    if from_decimals < to_decimals {
        let diff = to_decimals - from_decimals;
        let factor = BigUint::from(10u32).pow(diff);
        return Ok(value * factor);
    }

    let diff = from_decimals - to_decimals;
    let factor = BigUint::from(10u32).pow(diff);
    let remainder = &value % &factor;
    if !remainder.is_zero() {
        return Err(SwapperError::InvalidAmount("amount precision loss".to_string()));
    }

    Ok(value / factor)
}

pub fn scale_quote_value(value: &str, from_decimals: u32, to_decimals: u32) -> Result<String, SwapperError> {
    let amount = BigUint::from_str(value)?;
    let scaled = scale_units(amount, from_decimals, to_decimals)?;
    Ok(scaled.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn test_scale_units_increase_precision() {
        let base = BigUint::from(123u32);
        let scaled = scale_units(base.clone(), 8, 18).unwrap();
        let expected = BigUint::from(10u32).pow(10) * base;
        assert_eq!(scaled, expected);
    }

    #[test]
    fn test_scale_units_reduce_precision() {
        let value = BigUint::from(123u32) * BigUint::from(10u32).pow(10);
        let scaled = scale_units(value.clone(), 18, 8).unwrap();
        assert_eq!(scaled, BigUint::from(123u32));
    }

    #[test]
    fn test_scale_units_precision_loss_rejected() {
        let err = scale_units(BigUint::from(5u32), 3, 1).unwrap_err();
        assert!(matches!(err, SwapperError::InvalidAmount(_)));
    }

    #[test]
    fn test_scale_quote_value_increase_precision() {
        let result = scale_quote_value("123000000", 6, 8).unwrap();
        assert_eq!(result, "12300000000");
    }

    #[test]
    fn test_scale_quote_value_preserves_exact_division() {
        let result = scale_quote_value("12300000000", 8, 6).unwrap();
        assert_eq!(result, "123000000");
    }

    #[test]
    fn test_scale_quote_value_invalid_number() {
        let err = scale_quote_value("abc", 6, 8).unwrap_err();
        assert!(matches!(err, SwapperError::InvalidAmount(_)));
    }
}
