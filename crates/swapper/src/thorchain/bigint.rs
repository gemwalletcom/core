use num_bigint::BigInt;

use crate::SwapperError;

const THORCHAIN_BASE_DECIMALS: i32 = 8;

pub(crate) fn value_from(value: &str, decimals: i32) -> Result<BigInt, SwapperError> {
    let value = value.parse::<BigInt>()?;
    let decimals = decimals - THORCHAIN_BASE_DECIMALS;
    let factor = BigInt::from(10).pow(decimals.unsigned_abs());
    Ok(if decimals > 0 { value / factor } else { value * factor })
}

pub(crate) fn value_to(value: &str, decimals: i32) -> Result<BigInt, SwapperError> {
    let value = value.parse::<BigInt>()?;
    let decimals = decimals - THORCHAIN_BASE_DECIMALS;
    let factor = BigInt::from(10).pow(decimals.unsigned_abs());
    Ok(if decimals > 0 { value * factor } else { value / factor })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_from() {
        let value = "1000000000";

        let result = value_from(value, 18).unwrap();
        assert_eq!(result, BigInt::from(0));

        let result = value_from(value, 10).unwrap();
        assert_eq!(result, BigInt::from(10000000));

        let result = value_from(value, 6).unwrap();
        assert_eq!(result, BigInt::from(100000000000u64));

        let result = value_from(value, 8).unwrap();
        assert_eq!(result, BigInt::from(1000000000u64));
    }

    #[test]
    fn test_value_to() {
        let value = "10000000";

        let result = value_to(value, 18).unwrap();
        assert_eq!(result, BigInt::from(100000000000000000u64));

        let result = value_to(value, 10).unwrap();
        assert_eq!(result, BigInt::from(1000000000u64));

        let result = value_to(value, 6).unwrap();
        assert_eq!(result, BigInt::from(100000u64));

        let result = value_to(value, 8).unwrap();
        assert_eq!(result, BigInt::from(10000000u64));
    }
}
