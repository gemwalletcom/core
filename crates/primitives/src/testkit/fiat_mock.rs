use crate::currency::Currency;
use crate::fiat_assets::FiatAssetLimits;
use crate::{Asset, Chain, FiatBuyQuote, FiatProvider, FiatQuote, FiatQuoteRequest, FiatQuoteType, PaymentType};

impl FiatBuyQuote {
    pub fn mock() -> Self {
        FiatBuyQuote {
            asset: Asset::from_chain(Chain::Bitcoin),
            asset_id: Chain::Bitcoin.as_asset_id().to_string(),
            ip_address: "192.168.1.1".to_string(),
            fiat_currency: Currency::USD,
            fiat_amount: 100.0,
            fiat_value: "100.0".to_string(),
            wallet_address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
        }
    }
}

impl FiatQuoteRequest {
    pub fn mock() -> Self {
        FiatQuoteRequest {
            asset_id: Chain::Bitcoin.as_asset_id().to_string(),
            quote_type: FiatQuoteType::Buy,
            ip_address: "192.168.1.1".to_string(),
            fiat_currency: Currency::USD,
            fiat_amount: Some(100.0),
            crypto_value: None,
            wallet_address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
            provider_id: None,
        }
    }
}

impl FiatProvider {
    pub fn mock(id: &str) -> Self {
        FiatProvider {
            id: id.to_string(),
            name: id.to_string(),
            image_url: Some("".to_string()),
            priority: None,
            threshold_bps: None,
        }
    }

    pub fn mock_with_priority(id: &str, priority: i32, threshold_bps: Option<i32>) -> Self {
        FiatProvider {
            id: id.to_string(),
            name: id.to_string(),
            image_url: Some("".to_string()),
            priority: Some(priority),
            threshold_bps,
        }
    }
}

impl FiatQuote {
    pub fn mock(provider_id: &str, crypto_amount: f64, fiat_amount: f64) -> Self {
        FiatQuote {
            provider: FiatProvider::mock(provider_id),
            quote_type: FiatQuoteType::Buy,
            fiat_amount,
            fiat_currency: "USD".to_string(),
            crypto_amount,
            crypto_value: crypto_amount.to_string(),
            redirect_url: "".to_string(),
        }
    }
}

impl FiatAssetLimits {
    pub fn mock() -> Self {
        FiatAssetLimits {
            currency: Currency::USD,
            payment_type: PaymentType::Card,
            min_amount: Some(50.0),
            max_amount: Some(10000.0),
        }
    }

    pub fn mock_usd(min_amount: f64, max_amount: f64) -> Self {
        FiatAssetLimits {
            currency: Currency::USD,
            payment_type: PaymentType::Card,
            min_amount: Some(min_amount),
            max_amount: Some(max_amount),
        }
    }
}
