use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use std::str::FromStr;

pub struct BigNumberFormatter {}

impl BigNumberFormatter {
    pub fn big_decimal_value(value: &str, decimals: u32) -> Option<BigDecimal> {
        let mut decimal = BigDecimal::from_str(value).ok()?;
        let exp = BigInt::from(10).pow(decimals);
        decimal = decimal / BigDecimal::from(exp);
        Some(decimal)
    }
    pub fn value(value: &str, decimals: i32) -> Option<String> {
        let decimal = Self::big_decimal_value(value, decimals as u32)?;
        Some(decimal.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value() {
        // Test case 1: Valid input
        let result = BigNumberFormatter::value("123456", 3).unwrap();
        assert_eq!(result, "123.456");

        // Test case 2: Input with more decimals than specified
        let result = BigNumberFormatter::value("789123456", 4).unwrap();
        assert_eq!(result, "78912.3456");

        // Test case 3: Input with fewer decimals than specified
        let result = BigNumberFormatter::value("4567", 4).unwrap();
        assert_eq!(result, "0.4567");

        // Test case 4: u256 input
        let result = BigNumberFormatter::value(
            "115792089237316195423570985008687907853269984665640564039457000000000000000000",
            18,
        )
        .unwrap();
        assert_eq!(
            result,
            "115792089237316195423570985008687907853269984665640564039457"
        );

        // Test case 5: Invalid input
        let result = BigNumberFormatter::value("abc", 2);
        assert_eq!(result, None);

        // Test case 6: Output return small value
        let result = BigNumberFormatter::value("1640000000000000", 18).unwrap();
        assert_eq!(result, "0.00164");
    }
}
