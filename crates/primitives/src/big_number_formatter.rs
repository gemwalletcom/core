use bigdecimal::{BigDecimal, RoundingMode};
use num_bigint::BigInt;
use num_format::{Locale, ToFormattedString};
use std::str::FromStr;

pub struct BigNumberFormatter {}

impl BigNumberFormatter {
    fn get_formatted_scale(value: &str, decimals: i32, target_scale: i64) -> Option<i64> {
        let decimal = Self::big_decimal_value(value, decimals as u32)?;
        let decimal_string = decimal.to_string();

        let parts: Vec<&str> = decimal_string.split('.').collect();
        if parts.len() < 2 {
            // No fractional part => just use target_scale
            return Some(target_scale);
        }

        // Example: "0.00012" => integer_part="0", frac_str="00012"
        // Example: "-0.00012" => integer_part="-0", frac_str="00012"
        let mut frac_str = parts[1];

        // If negative, strip the leading '-'.
        // Usually `parts[1]` won't have '-', but just in case we do this.
        if frac_str.starts_with('-') {
            frac_str = &frac_str[1..];
        }

        // Count how many '0' from the start of fractional part until first non-'0'
        let mut leading_zero_count = 0;
        for ch in frac_str.chars() {
            if ch == '0' {
                leading_zero_count += 1;
            } else {
                break;
            }
        }

        let computed_scale = leading_zero_count as i64 + target_scale;
        if computed_scale > frac_str.len() as i64 {
            // If our desired scale surpasses the entire fractional length,
            // we fallback to just 'leading_zero_count'.
            // This ensures we don't try to keep beyond the available digits.
            Some(leading_zero_count as i64)
        } else {
            Some(computed_scale)
        }
    }

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

    pub fn localized_value_with_scale(value: &str, decimals: i32, target_scale: i64, locale: Option<Locale>) -> Option<String> {
        let scale = Self::get_formatted_scale(value, decimals, target_scale)?;
        let decimal = Self::big_decimal_value(value, decimals as u32)?;
        let rounded_decimal = decimal.with_scale_round(scale, RoundingMode::Ceiling);

        let s = rounded_decimal.to_string();
        let parts: Vec<&str> = s.split('.').collect();
        let integer_part = parts[0];
        let fractional_part = if parts.len() > 1 { format!(".{}", parts[1]) } else { String::new() };

        // Convert integer_part to a BigInt for formatting with `num-format`.
        // Handle negative numbers.
        let is_negative = integer_part.starts_with('-');
        let abs_str = integer_part.trim_start_matches('-');
        let int_val = abs_str.parse::<i128>().unwrap_or(0);

        // Format the absolute value with commas.
        let formatted_abs = int_val.to_formatted_string(&locale.unwrap_or(Locale::en));

        let result = if is_negative {
            format!("-{}{}", formatted_abs, fractional_part)
        } else {
            format!("{}{}", formatted_abs, fractional_part)
        };

        Some(result)
    }

    pub fn value_from_amount(amount: &str, decimals: u32) -> Option<String> {
        let big_decimal = BigDecimal::from_str(amount).expect("Invalid decimal number");
        let multiplier = BigInt::from(10).pow(decimals);
        let multiplier_decimal = BigDecimal::from(multiplier);
        let scaled_value = big_decimal * multiplier_decimal;
        Some(scaled_value.with_scale(0).to_string())
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
        assert_eq!(result, None);

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
    }

    #[test]
    fn test_format_with_commas() {
        let locale = Locale::en;
        assert_eq!(
            BigNumberFormatter::localized_value_with_scale("1123450000", 6, 2, Some(locale)).unwrap(),
            "1,123.45"
        );
        assert_eq!(
            BigNumberFormatter::localized_value_with_scale("1123456666", 6, 2, Some(locale)).unwrap(),
            "1,123.46"
        );

        assert_eq!(BigNumberFormatter::localized_value_with_scale("12000", 8, 2, Some(locale)).unwrap(), "0.00012");
        assert_eq!(BigNumberFormatter::localized_value_with_scale("129999", 8, 2, Some(locale)).unwrap(), "0.0013");
    }

    #[test]
    fn test_get_formatted_scale() {
        assert_eq!(BigNumberFormatter::get_formatted_scale("123450000", 6, 2).unwrap(), 2);
        assert_eq!(BigNumberFormatter::get_formatted_scale("123456666", 6, 2).unwrap(), 2);
        assert_eq!(BigNumberFormatter::get_formatted_scale("12000", 8, 2).unwrap(), 5);
        assert_eq!(BigNumberFormatter::get_formatted_scale("129999", 8, 2).unwrap(), 4);
    }
}
