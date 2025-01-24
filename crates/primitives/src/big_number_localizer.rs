use crate::BigNumberFormatter;
use bigdecimal::RoundingMode;
use num_format::{Locale, ToFormattedString};

#[derive(Default)]
pub struct BigNumberLocalizer {}

pub enum Format {
    Short,
    Medium,
}

impl Format {
    fn get_scale(&self) -> i64 {
        match self {
            Format::Short => 2,
            Format::Medium => 4,
        }
    }
}

impl BigNumberLocalizer {
    pub fn get_value(&self, value: &str, decimals: i32, format: Format, locale: Locale) -> Option<String> {
        Self::localized_value_with_scale(value, decimals, format.get_scale(), locale)
    }

    fn get_formatted_scale(value: &str, decimals: i32, target_scale: i64) -> Option<i64> {
        let decimal = BigNumberFormatter::big_decimal_value(value, decimals as u32)?;
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

    fn localized_value_with_scale(value: &str, decimals: i32, target_scale: i64, locale: Locale) -> Option<String> {
        let scale = Self::get_formatted_scale(value, decimals, target_scale)?;
        let decimal = BigNumberFormatter::big_decimal_value(value, decimals as u32)?;
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
        let formatted_abs = int_val.to_formatted_string(&locale);

        let result = if is_negative {
            format!("-{}{}", formatted_abs, fractional_part)
        } else {
            format!("{}{}", formatted_abs, fractional_part)
        };

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_with_commas() {
        let localizer = BigNumberLocalizer::default();
        let locale = Locale::en;
        assert_eq!(localizer.get_value("1123450000", 0, Format::Short, locale).unwrap(), "1,123,450,000.00");
        assert_eq!(localizer.get_value("1123450000", 6, Format::Short, locale).unwrap(), "1,123.45");
        assert_eq!(localizer.get_value("1123450000", 6, Format::Medium, locale).unwrap(), "1,124"); // Invalid?
        assert_eq!(localizer.get_value("1123456666", 6, Format::Short, locale).unwrap(), "1,123.46");

        assert_eq!(localizer.get_value("12000", 8, Format::Short, locale).unwrap(), "0.00012");
        assert_eq!(localizer.get_value("0", 8, Format::Short, locale).unwrap(), "0.00");
        assert_eq!(localizer.get_value("1", 8, Format::Short, locale).unwrap(), "0.01"); // Invalid?
        assert_eq!(localizer.get_value("1", 8, Format::Medium, locale).unwrap(), "0.0001"); // Invalid?
        assert_eq!(localizer.get_value("129999", 8, Format::Short, locale).unwrap(), "0.0013");
        // Invalid
    }

    #[test]
    fn test_get_formatted_scale() {
        assert_eq!(BigNumberLocalizer::get_formatted_scale("123450000", 6, 2).unwrap(), 2);
        assert_eq!(BigNumberLocalizer::get_formatted_scale("123456666", 6, 2).unwrap(), 2);
        assert_eq!(BigNumberLocalizer::get_formatted_scale("12000", 8, 2).unwrap(), 5);
        assert_eq!(BigNumberLocalizer::get_formatted_scale("129999", 8, 2).unwrap(), 4);
    }
}
