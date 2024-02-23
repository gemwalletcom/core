use rust_decimal::{prelude::*, Decimal};
pub struct NumberFormatter {}

impl NumberFormatter {
    pub fn value(value: &str, decimals: i32) -> Option<String> {
        let mut crypto_amount: Decimal = Decimal::from_str(value).ok()?;
        crypto_amount.set_scale(decimals as u32).ok()?;
        let amount: f64 = crypto_amount.to_f64()?;
        format!("{}", amount).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value() {
        // Test case 1: Valid input
        let result = NumberFormatter::value("123456", 3);
        assert_eq!(result, Some("123.456".to_string()));

        // Test case 2: Input with more decimals than specified
        let result = NumberFormatter::value("789123456", 4);
        assert_eq!(result, Some("78912.3456".to_string()));

        // Test case 3: Input with fewer decimals than specified
        let result = NumberFormatter::value("4567", 4);
        assert_eq!(result, Some("0.4567".to_string()));

        // Test case 4: Invalid input
        let result = NumberFormatter::value("abc", 2);
        assert_eq!(result, None);
    }
}
