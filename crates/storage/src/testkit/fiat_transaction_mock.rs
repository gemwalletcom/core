use crate::models::FiatTransactionRow;
use chrono::DateTime;
use primitives::{AssetId, Chain, FiatProviderName, FiatQuoteType, FiatTransactionStatus};

impl FiatTransactionRow {
    pub fn mock() -> Self {
        Self {
            id: 1,
            asset_id: AssetId::from_chain(Chain::Ethereum).into(),
            transaction_type: FiatQuoteType::Buy.into(),
            provider_id: FiatProviderName::MoonPay.into(),
            provider_transaction_id: Some("tx_123".to_string()),
            status: FiatTransactionStatus::Pending.into(),
            country: Some("US".to_string()),
            fiat_amount: 100.0,
            fiat_currency: "USD".to_string(),
            value: Some("123000000000000000".to_string()),
            address: Some("0x123".to_string()),
            transaction_hash: Some("0xabc".to_string()),
            device_id: 1,
            wallet_id: 1,
            quote_id: "quote_123".to_string(),
            updated_at: DateTime::UNIX_EPOCH.naive_utc(),
            created_at: DateTime::UNIX_EPOCH.naive_utc(),
        }
    }

    pub fn mock_with_timestamps(created_at: DateTime<chrono::Utc>, updated_at: DateTime<chrono::Utc>) -> Self {
        Self {
            created_at: created_at.naive_utc(),
            updated_at: updated_at.naive_utc(),
            ..Self::mock()
        }
    }

    pub fn mock_without_value() -> Self {
        Self { value: None, ..Self::mock() }
    }
}
