use hex;
use primitives::FiatQuoteType;
use sha2::{Digest, Sha512};
use url::Url;

use super::models::Quote;

const MERCURYO_REDIRECT_URL: &str = "https://exchange.mercuryo.io";

pub struct MercuryoWidget {
    widget_id: String,
    secret_key: String,
    merchant_transaction_id: String,
    address: String,
    ip_address: String,
    currency: String,
    network: String,
    quote_type: FiatQuoteType,
    amount: f64,
}

impl MercuryoWidget {
    pub fn new(widget_id: String, secret_key: String, address: String, ip_address: String, quote: Quote, quote_type: FiatQuoteType, network: String) -> Self {
        let amount = match quote_type {
            FiatQuoteType::Buy => quote.fiat_amount,
            FiatQuoteType::Sell => quote.amount,
        };

        Self {
            widget_id,
            secret_key,
            merchant_transaction_id: uuid::Uuid::new_v4().to_string(),
            address,
            ip_address,
            currency: quote.currency,
            network,
            quote_type,
            amount,
        }
    }

    pub fn new_from_data(
        widget_id: String,
        secret_key: String,
        merchant_transaction_id: String,
        address: String,
        ip_address: String,
        currency: String,
        _fiat_currency: String,
        amount: f64,
        quote_type: FiatQuoteType,
        network: String,
    ) -> Self {
        Self {
            widget_id,
            secret_key,
            merchant_transaction_id,
            address,
            ip_address,
            currency,
            network,
            quote_type,
            amount,
        }
    }

    fn signature(&self) -> String {
        let content = format!("{}{}{}{}", self.address, self.secret_key, self.ip_address, self.merchant_transaction_id);
        let hash = hex::encode(Sha512::digest(content));
        format!("v2:{}", hash)
    }

    pub fn merchant_transaction_id(&self) -> &str {
        &self.merchant_transaction_id
    }

    pub fn to_url(&self) -> String {
        let mut url = Url::parse(MERCURYO_REDIRECT_URL).unwrap();

        url.query_pairs_mut()
            .append_pair("widget_id", &self.widget_id)
            .append_pair("merchant_transaction_id", &self.merchant_transaction_id)
            .append_pair("currency", &self.currency)
            .append_pair("address", &self.address)
            .append_pair("network", &self.network)
            .append_pair("signature", &self.signature());

        match self.quote_type {
            FiatQuoteType::Buy => {
                url.query_pairs_mut()
                    .append_pair("type", "buy")
                    .append_pair("fiat_amount", &self.amount.to_string());
            }
            FiatQuoteType::Sell => {
                url.query_pairs_mut()
                    .append_pair("type", "sell")
                    .append_pair("amount", &self.amount.to_string());
            }
        };

        url.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_v2_format() {
        let quote = Quote {
            amount: 0.5,
            currency: "BTC".to_string(),
            fiat_amount: 1000.0,
        };
        let mut widget = MercuryoWidget::new(
            "widget123".to_string(),
            "secret".to_string(),
            "0x123".to_string(),
            "127.0.0.1".to_string(),
            quote,
            FiatQuoteType::Buy,
            "BITCOIN".to_string(),
        );
        widget.merchant_transaction_id = "tx123".to_string();

        let signature = widget.signature();
        let expected_content = "0x123secret127.0.0.1tx123";
        let expected_hash = hex::encode(Sha512::digest(expected_content));

        assert_eq!(signature, format!("v2:{}", expected_hash));
    }

    #[test]
    fn build_url_buy() {
        let quote = Quote {
            amount: 0.5,
            currency: "BTC".to_string(),
            fiat_amount: 1000.0,
        };

        let widget = MercuryoWidget::new(
            "widget123".to_string(),
            "secret".to_string(),
            "0x123".to_string(),
            "127.0.0.1".to_string(),
            quote,
            FiatQuoteType::Buy,
            "BITCOIN".to_string(),
        );
        let url = widget.to_url();

        assert!(url.starts_with("https://exchange.mercuryo.io"));
        assert!(url.contains("widget_id=widget123"));
        assert!(url.contains("currency=BTC"));
        assert!(url.contains("address=0x123"));
        assert!(url.contains("network=BITCOIN"));
        assert!(url.contains("type=buy"));
        assert!(url.contains("fiat_amount=1000"));
        assert!(url.contains("signature=v2%3A"));
    }

    #[test]
    fn build_url_sell() {
        let quote = Quote {
            amount: 1.5,
            currency: "ETH".to_string(),
            fiat_amount: 3000.0,
        };

        let widget = MercuryoWidget::new(
            "widget123".to_string(),
            "secret".to_string(),
            "0xdef".to_string(),
            "127.0.0.1".to_string(),
            quote,
            FiatQuoteType::Sell,
            "ETHEREUM".to_string(),
        );
        let url = widget.to_url();

        assert!(url.contains("type=sell"));
        assert!(url.contains("amount=1.5"));
        assert!(!url.contains("fiat_amount="));
    }
}
