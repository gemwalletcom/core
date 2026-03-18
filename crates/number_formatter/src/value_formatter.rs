use bigdecimal::BigDecimal;
use num_bigint::Sign;

use crate::big_number_formatter::{BigNumberFormatter, NumberFormatterError};
use crate::currency::add_thousands_separator;

pub enum ValueStyle {
    Full,
    Auto,
}

pub struct ValueFormatter;

impl ValueFormatter {
    pub fn format(style: ValueStyle, value: &str, decimals: i32) -> Result<String, NumberFormatterError> {
        let decimal = BigNumberFormatter::big_decimal_value(value, decimals as u32)?;
        match style {
            ValueStyle::Full => Ok(format_full(&decimal)),
            ValueStyle::Auto => Ok(format_auto(&decimal)),
        }
    }

    pub fn format_f64(style: ValueStyle, value: f64) -> String {
        let decimal: BigDecimal = value.to_string().parse().unwrap_or_default();
        match style {
            ValueStyle::Full => format_full(&decimal),
            ValueStyle::Auto => format_auto(&decimal),
        }
    }

    pub fn format_f64_currency(style: ValueStyle, value: f64, symbol: &str) -> String {
        format!("{}{}", symbol, Self::format_f64(style, value))
    }

    pub fn format_with_symbol(style: ValueStyle, value: &str, decimals: i32, symbol: &str) -> Result<String, NumberFormatterError> {
        let formatted = Self::format(style, value, decimals)?;
        Ok(format!("{} {}", formatted, symbol))
    }
}

fn bigdecimal_to_plain_string(decimal: &BigDecimal) -> String {
    let (bigint, scale) = decimal.as_bigint_and_exponent();
    let is_negative = bigint.sign() == Sign::Minus;
    let digits = bigint.magnitude().to_string();

    let result = if scale <= 0 {
        let zeros = "0".repeat((-scale) as usize);
        format!("{}{}", digits, zeros)
    } else {
        let scale = scale as usize;
        if digits.len() <= scale {
            let zeros = "0".repeat(scale - digits.len());
            format!("0.{}{}", zeros, digits)
        } else {
            let (integer, fraction) = digits.split_at(digits.len() - scale);
            format!("{}.{}", integer, fraction)
        }
    };

    if is_negative { format!("-{}", result) } else { result }
}

fn format_full(decimal: &BigDecimal) -> String {
    let plain = bigdecimal_to_plain_string(&decimal.normalized());
    let plain = strip_trailing_zeros(&plain);
    apply_thousands_separator(&plain)
}

fn format_auto(decimal: &BigDecimal) -> String {
    if decimal.sign() == Sign::NoSign {
        return "0".to_string();
    }

    let abs = decimal.abs();
    let one = BigDecimal::from(1);
    let threshold = BigDecimal::new(1.into(), 4); // 0.0001

    if abs >= one {
        format_short(decimal)
    } else if abs >= threshold {
        format_middle(decimal)
    } else {
        format_full(decimal)
    }
}

fn format_short(decimal: &BigDecimal) -> String {
    let plain = bigdecimal_to_plain_string(decimal);
    let (integer, fraction) = plain.split_once('.').unwrap_or((&plain, ""));
    let fraction = if fraction.len() > 2 { &fraction[..2] } else { fraction };
    apply_thousands_separator(&format!("{}.{:0<2}", integer, fraction))
}

fn format_middle(decimal: &BigDecimal) -> String {
    let plain = bigdecimal_to_plain_string(&decimal.normalized());
    truncate_significant(&plain, 4)
}

fn truncate_significant(value_str: &str, max_sig: usize) -> String {
    let (is_negative, abs_str) = if let Some(stripped) = value_str.strip_prefix('-') {
        (true, stripped)
    } else {
        (false, value_str)
    };

    let (_, fraction) = abs_str.split_once('.').unwrap_or((abs_str, ""));
    let leading_zeros = fraction.chars().take_while(|&c| c == '0').count();
    let sig_end = (leading_zeros + max_sig).min(fraction.len());

    let prefix = if is_negative { "-" } else { "" };
    format!("{}0.{}", prefix, &fraction[..sig_end])
}

fn strip_trailing_zeros(value: &str) -> String {
    if !value.contains('.') {
        return value.to_string();
    }
    let trimmed = value.trim_end_matches('0');
    if let Some(stripped) = trimmed.strip_suffix('.') {
        stripped.to_string()
    } else {
        trimmed.to_string()
    }
}

fn apply_thousands_separator(value: &str) -> String {
    add_thousands_separator(value, ',', '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_style() {
        assert_eq!(ValueFormatter::format(ValueStyle::Full, "123", 0).unwrap(), "123");
        assert_eq!(ValueFormatter::format(ValueStyle::Full, "0", 0).unwrap(), "0");
        assert_eq!(ValueFormatter::format(ValueStyle::Full, "1000000", 0).unwrap(), "1,000,000");
        assert_eq!(ValueFormatter::format(ValueStyle::Full, "12344", 6).unwrap(), "0.012344");
        assert_eq!(ValueFormatter::format(ValueStyle::Full, "1", 4).unwrap(), "0.0001");
        assert_eq!(ValueFormatter::format(ValueStyle::Full, "1", 6).unwrap(), "0.000001");
        assert_eq!(ValueFormatter::format(ValueStyle::Full, "12345678910111213", 18).unwrap(), "0.012345678910111213");
        assert_eq!(ValueFormatter::format(ValueStyle::Full, "1", 18).unwrap(), "0.000000000000000001");
        assert_eq!(ValueFormatter::format(ValueStyle::Full, "18761627355200464162", 18).unwrap(), "18.761627355200464162");
        assert_eq!(ValueFormatter::format(ValueStyle::Full, "4162", 18).unwrap(), "0.000000000000004162");
    }

    #[test]
    fn test_full_with_symbol() {
        assert_eq!(ValueFormatter::format_with_symbol(ValueStyle::Full, "2737071", 8, "BTC").unwrap(), "0.02737071 BTC");
    }

    #[test]
    fn test_auto_style() {
        assert_eq!(ValueFormatter::format(ValueStyle::Auto, "123", 0).unwrap(), "123.00");
        assert_eq!(ValueFormatter::format(ValueStyle::Auto, "1000000", 0).unwrap(), "1,000,000.00");
        assert_eq!(ValueFormatter::format(ValueStyle::Auto, "18761627355200464162", 18).unwrap(), "18.76");
        assert_eq!(ValueFormatter::format(ValueStyle::Auto, "12344", 6).unwrap(), "0.01234");
        assert_eq!(ValueFormatter::format(ValueStyle::Auto, "11112344", 10).unwrap(), "0.001111");
        assert_eq!(ValueFormatter::format(ValueStyle::Auto, "1", 4).unwrap(), "0.0001");
        assert_eq!(ValueFormatter::format(ValueStyle::Auto, "1", 5).unwrap(), "0.00001");
        assert_eq!(ValueFormatter::format(ValueStyle::Auto, "4162", 18).unwrap(), "0.000000000000004162");
        assert_eq!(ValueFormatter::format(ValueStyle::Auto, "0", 0).unwrap(), "0");
    }

    #[test]
    fn test_invalid_input() {
        assert!(ValueFormatter::format(ValueStyle::Auto, "abc", 0).is_err());
    }

    #[test]
    fn test_format_f64() {
        assert_eq!(ValueFormatter::format_f64(ValueStyle::Auto, 25432.50), "25,432.50");
        assert_eq!(ValueFormatter::format_f64(ValueStyle::Auto, 0.0), "0");
        assert_eq!(ValueFormatter::format_f64(ValueStyle::Auto, 1.5), "1.50");
        assert_eq!(ValueFormatter::format_f64(ValueStyle::Auto, 100000.0), "100,000.00");
        assert_eq!(ValueFormatter::format_f64(ValueStyle::Auto, 0.005), "0.005");
        assert_eq!(ValueFormatter::format_f64(ValueStyle::Auto, -123.45), "-123.45");
        assert_eq!(ValueFormatter::format_f64(ValueStyle::Auto, -1500.0), "-1,500.00");
        assert_eq!(ValueFormatter::format_f64(ValueStyle::Full, -123.456), "-123.456");
    }

    #[test]
    fn test_format_f64_currency() {
        assert_eq!(ValueFormatter::format_f64_currency(ValueStyle::Auto, 25432.50, "$"), "$25,432.50");
        assert_eq!(ValueFormatter::format_f64_currency(ValueStyle::Auto, -123.45, "$"), "$-123.45");
        assert_eq!(ValueFormatter::format_f64_currency(ValueStyle::Auto, 0.0, "$"), "$0");
    }
}
