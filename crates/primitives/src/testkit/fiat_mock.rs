use crate::currency::Currency;
use crate::fiat_assets::FiatAssetLimits;
use crate::{Asset, Chain, FiatBuyQuote, FiatQuoteRequest, FiatQuoteType, PaymentType};

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
