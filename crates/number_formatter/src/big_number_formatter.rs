use bigdecimal::{BigDecimal, ToPrimitive};
use num_bigint::{BigInt, BigUint};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum NumberFormatterError {
    InvalidNumber(String),
    ConversionError(String),
}

impl std::fmt::Display for NumberFormatterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNumber(msg) => write!(f, "Invalid number: {}", msg),
            Self::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
        }
    }
}

impl std::error::Error for NumberFormatterError {}

impl From<NumberFormatterError> for String {
    fn from(error: NumberFormatterError) -> Self {
        error.to_string()
    }
}

pub struct BigNumberFormatter {}

impl BigNumberFormatter {
    pub fn big_decimal_value(value: &str, decimals: u32) -> Result<BigDecimal, NumberFormatterError> {
        let mut decimal = BigDecimal::from_str(value).map_err(|e| NumberFormatterError::InvalidNumber(e.to_string()))?;
        let exp = BigInt::from(10).pow(decimals);
        decimal = decimal / BigDecimal::from(exp);
        Ok(decimal)
    }

    pub fn value_as_f64(value: &str, decimals: u32) -> Result<f64, NumberFormatterError> {
        Self::big_decimal_value(value, decimals)?
            .to_f64()
            .ok_or_else(|| NumberFormatterError::ConversionError("Cannot convert to f64".to_string()))
    }

    pub fn value_as_u64(value: &str, decimals: u32) -> Result<u64, NumberFormatterError> {
        Self::big_decimal_value(value, decimals)?
            .to_u64()
            .ok_or_else(|| NumberFormatterError::ConversionError("Cannot convert to u64".to_string()))
    }

    pub fn value(value: &str, decimals: i32) -> Result<String, NumberFormatterError> {
        let decimal = Self::big_decimal_value(value, decimals as u32)?;
        Ok(decimal.to_string())
    }

    pub fn value_from_amount(amount: &str, decimals: u32) -> Result<String, String> {
        let big_decimal = BigDecimal::from_str(amount).map_err(|_| "Invalid decimal number".to_string())?;
        let multiplier = BigInt::from(10).pow(decimals);
        let multiplier_decimal = BigDecimal::from(multiplier);
        let scaled_value = big_decimal * multiplier_decimal;
        Ok(scaled_value.with_scale(0).to_string())
    }

    pub fn f64_as_value(amount: f64, decimals: u32) -> Option<String> {
        Self::value_from_amount(&amount.to_string(), decimals).ok()
    }

    pub fn value_from_amount_biguint(amount: &str, decimals: u32) -> Result<BigUint, String> {
        let big_decimal = BigDecimal::from_str(amount).map_err(|_| "Invalid decimal number".to_string())?;
        let multiplier = BigInt::from(10).pow(decimals);
        let multiplier_decimal = BigDecimal::from(multiplier);
        let scaled_value = big_decimal * multiplier_decimal;
        let scaled_string = scaled_value.with_scale(0).to_string();
        scaled_string.parse::<BigUint>().map_err(|_| "Cannot convert to BigUint".to_string())
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
        let result = BigNumberFormatter::value("115792089237316195423570985008687907853269984665640564039457000000000000000000", 18).unwrap();
        assert_eq!(result, "115792089237316195423570985008687907853269984665640564039457");

        // Test case 5: Invalid input
        let result = BigNumberFormatter::value("abc", 2);
        assert!(result.is_err());

        // Test case 6: Output return small value
        let result = BigNumberFormatter::value("1640000000000000", 18).unwrap();
        assert_eq!(result, "0.00164");
    }

    #[test]
    fn test_value_from_amount() {
        // Test case 1: Valid input
        let result = BigNumberFormatter::value_from_amount("1.123", 3).unwrap();
        assert_eq!(result, "1123");

        let result = BigNumberFormatter::value_from_amount("332131212.2321312", 8).unwrap();
        assert_eq!(result, "33213121223213120");

        let result = BigNumberFormatter::value_from_amount("0", 0).unwrap();
        assert_eq!(result, "0");

        // Test case 2: Invalid input
        let result = BigNumberFormatter::value_from_amount("invalid", 3);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid decimal number");
    }

    #[test]
    fn test_value_from_amount_biguint() {
        // Test case 1: Valid input
        let result = BigNumberFormatter::value_from_amount_biguint("1.123", 3).unwrap();
        assert_eq!(result, BigUint::from(1123u32));

        let result = BigNumberFormatter::value_from_amount_biguint("332131212.2321312", 8).unwrap();
        assert_eq!(result, BigUint::from(33213121223213120_u64));

        let result = BigNumberFormatter::value_from_amount_biguint("0", 0).unwrap();
        assert_eq!(result, BigUint::from(0u32));

        // Test case 2: Large numbers
        let result = BigNumberFormatter::value_from_amount_biguint("1000000000000", 18).unwrap();
        let expected = "1000000000000000000000000000000".parse::<BigUint>().unwrap();
        assert_eq!(result, expected);

        // Test case 3: Invalid input
        let result = BigNumberFormatter::value_from_amount_biguint("invalid", 3);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid decimal number");
    }
}
