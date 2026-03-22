use crate::currency::Currency;
use crate::fiat_assets::FiatAssetLimits;
use crate::{
    Asset, AssetId, Chain, FiatProvider, FiatProviderName, FiatQuote, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteType, FiatTransaction, FiatTransactionStatus,
    FiatTransactionUpdate, PaymentType,
};
use chrono::{DateTime, Utc};

impl FiatQuoteRequest {
    pub fn mock() -> Self {
        FiatQuoteRequest {
            asset_id: AssetId::from_chain(Chain::Bitcoin),
            quote_type: FiatQuoteType::Buy,
            currency: "USD".to_string(),
            amount: 100.0,
            provider_id: None,
            ip_address: "192.168.1.1".to_string(),
        }
    }

    pub fn mock_sell() -> Self {
        FiatQuoteRequest {
            asset_id: AssetId::from_chain(Chain::Bitcoin),
            quote_type: FiatQuoteType::Sell,
            currency: "USD".to_string(),
            amount: 250.0,
            provider_id: None,
            ip_address: "192.168.1.1".to_string(),
        }
    }
}

impl FiatProvider {
    pub fn mock(id: FiatProviderName) -> Self {
        FiatProvider {
            id,
            name: id.name().to_string(),
            image_url: None,
            priority: None,
            threshold_bps: None,
            enabled: true,
            buy_enabled: true,
            sell_enabled: true,
            payment_methods: vec![],
        }
    }

    pub fn mock_with_priority(id: FiatProviderName, priority: i32, threshold_bps: Option<i32>) -> Self {
        FiatProvider {
            id,
            name: id.name().to_string(),
            image_url: None,
            priority: Some(priority),
            threshold_bps,
            enabled: true,
            buy_enabled: true,
            sell_enabled: true,
            payment_methods: vec![],
        }
    }
}

impl FiatQuote {
    pub fn mock(provider_id: FiatProviderName) -> Self {
        FiatQuote {
            id: "quote_123".to_string(),
            asset: Asset::from_chain(Chain::Bitcoin),
            provider: FiatProvider::mock(provider_id),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: 100.0,
            fiat_currency: "USD".to_string(),
            crypto_amount: 0.001,
            value: "100000".to_string(),
            latency: 0,
            payment_methods: vec![PaymentType::Card],
        }
    }
}

impl FiatQuoteResponse {
    pub fn mock(quote_id: &str, crypto_amount: f64, fiat_amount: f64) -> Self {
        FiatQuoteResponse {
            quote_id: quote_id.to_string(),
            fiat_amount,
            crypto_amount,
            payment_methods: vec![],
        }
    }
}

impl FiatTransaction {
    pub fn mock() -> Self {
        FiatTransaction {
            asset_id: AssetId::from_chain(Chain::Bitcoin),
            transaction_type: FiatQuoteType::Buy,
            provider_id: FiatProviderName::MoonPay,
            provider_transaction_id: Some("tx_123".to_string()),
            status: FiatTransactionStatus::Pending,
            country: Some("US".to_string()),
            fiat_amount: 100.0,
            fiat_currency: "USD".to_string(),
            value: "100000".to_string(),
            transaction_hash: None,
            address: Some("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string()),
            created_at: DateTime::<Utc>::UNIX_EPOCH,
            updated_at: DateTime::<Utc>::UNIX_EPOCH,
        }
    }
}

impl FiatTransactionUpdate {
    pub fn mock() -> Self {
        FiatTransactionUpdate {
            transaction_id: "quote_123".to_string(),
            provider_transaction_id: Some("tx_123".to_string()),
            status: FiatTransactionStatus::Pending,
            transaction_hash: None,
            address: Some("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string()),
            fiat_amount: Some(100.0),
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
