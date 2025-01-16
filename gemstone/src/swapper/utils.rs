use std::str::FromStr;

use num_bigint::BigInt;

pub mod converter {
    use super::*;

    pub fn value_from(value: String, decimals: i32) -> BigInt {
        let decimals = decimals - 8;
        if decimals > 0 {
            BigInt::from_str(value.as_str()).unwrap() / BigInt::from(10).pow(decimals as u32)
        } else {
            BigInt::from_str(value.as_str()).unwrap() * BigInt::from(10).pow(decimals.unsigned_abs())
        }
    }

    pub fn value_to(value: String, decimals: i32) -> BigInt {
        let decimals = decimals - 8;
        if decimals > 0 {
            BigInt::from_str(value.as_str()).unwrap() * BigInt::from(10).pow((decimals).unsigned_abs())
        } else {
            BigInt::from_str(value.as_str()).unwrap() / BigInt::from(10).pow((decimals).unsigned_abs())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::converter;
    use num_bigint::BigInt;
    use std::str::FromStr;

    #[test]
    fn test_value_from() {
        let value = "1000000000".to_string();

        let result = converter::value_from(value.clone(), 18);
        assert_eq!(result, BigInt::from_str("0").unwrap());

        let result = converter::value_from(value.clone(), 10);
        assert_eq!(result, BigInt::from_str("10000000").unwrap());

        let result = converter::value_from(value.clone(), 6);
        assert_eq!(result, BigInt::from_str("100000000000").unwrap());

        let result = converter::value_from(value.clone(), 8);
        assert_eq!(result, BigInt::from(1000000000));
    }

    #[test]
    fn test_value_to() {
        let value = "10000000".to_string();

        let result = converter::value_to(value.clone(), 18);
        assert_eq!(result, BigInt::from_str("100000000000000000").unwrap());

        let result = converter::value_to(value.clone(), 10);
        assert_eq!(result, BigInt::from(1000000000));

        let result = converter::value_to(value.clone(), 6);
        assert_eq!(result, BigInt::from(100000));

        let result = converter::value_to(value.clone(), 8);
        assert_eq!(result, BigInt::from(10000000));
    }
}
