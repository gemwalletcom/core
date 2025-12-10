use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransakQuote {
    pub quote_id: String,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub crypto_currency: String,
    pub crypto_amount: f64,
    pub network: String,
    pub conversion_price: f64,
    pub total_fee: f64,
}

impl TransakQuote {
    pub fn sell_crypto_amount(&self, fiat_amount: f64) -> f64 {
        (fiat_amount + self.total_fee) * self.conversion_price
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sell_crypto_amount() {
        let quote = TransakQuote {
            quote_id: "test".to_string(),
            fiat_amount: 100.0,
            fiat_currency: "USD".to_string(),
            crypto_currency: "ETH".to_string(),
            crypto_amount: 0.03,
            network: "ethereum".to_string(),
            conversion_price: 0.0005,
            total_fee: 5.0,
        };

        // (100 + 5) * 0.0005 = 0.0525
        assert_eq!(quote.sell_crypto_amount(100.0), 0.0525);
    }
}
