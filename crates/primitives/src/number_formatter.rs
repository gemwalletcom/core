use rusty_money::{iso, Formatter, Money, Params};

pub struct NumberFormatter {}

impl NumberFormatter {
    pub fn new() -> Self {
        NumberFormatter {}
    }

    pub fn currency(&self, value: f64, currency: &str) -> Option<String> {
        let money = Money::from_str(value.to_string().as_str(), iso::USD).ok()?;
        let iso_currency = iso::find(&currency).unwrap_or(iso::USD);

        let rounding = if value > 1.0 { 2 } else { 4 };

        let params = Params {
            symbol: Some(iso_currency.symbol),
            code: Some(iso_currency.iso_alpha_code),
            rounding: Some(rounding),
            ..Default::default()
        };
        return Some(Formatter::money(&money, params));
    }

    pub fn percent(&self, value: f64, _locale: &str) -> String {
        return format!("{:.2}%", value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency() {
        let formatter = NumberFormatter::new();
        assert_eq!(formatter.currency(1000.0, "USD"), Some("$1,000.00".to_string()));
        assert_eq!(formatter.currency(60127.9263, "USD"), Some("$60,127.93".to_string()));
        assert_eq!(formatter.currency(0.123456, "USD"), Some("$0.1235".to_string()));
        assert_eq!(formatter.currency(9999.99, "USD"), Some("$9,999.99".to_string()));
        assert_eq!(formatter.currency(9999.99, "EUR"), Some("€9,999.99".to_string()));
        assert_eq!(formatter.currency(9999.99, "CNY"), Some("¥9,999.99".to_string()));
        assert_eq!(formatter.currency(01.99, "GBP"), Some("£1.99".to_string()));
        assert_eq!(formatter.currency(19.01, "JPY"), Some("¥19.01".to_string()));
        assert_eq!(formatter.currency(0.39, "USD"), Some("$0.39".to_string()));
        assert_eq!(formatter.currency(0.0039, "USD"), Some("$0.0039".to_string()));
        assert_eq!(formatter.currency(69.420, "USD"), Some("$69.42".to_string()));
    }

    #[test]
    fn test_number() {
        let formatter = NumberFormatter::new();
        assert_eq!(formatter.percent(0.12, "en"), "0.12%");
        assert_eq!(formatter.percent(129.99, "en"), "129.99%");
    }
}
