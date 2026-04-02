use crate::{Asset, AssetId, FiatProviderName, FiatQuoteUrlData};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct FiatTransaction {
    pub id: String,
    pub asset_id: AssetId,
    pub transaction_type: FiatQuoteType,
    pub provider: FiatProviderName,
    #[typeshare(skip)]
    #[serde(skip_serializing)]
    pub provider_transaction_id: Option<String>,
    pub status: FiatTransactionStatus,
    #[typeshare(skip)]
    #[serde(skip_serializing)]
    pub country: Option<String>,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub value: String,
    #[typeshare(skip)]
    #[serde(skip_serializing)]
    pub transaction_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    #[typeshare(skip)]
    #[serde(skip_serializing)]
    pub updated_at: DateTime<Utc>,
}

impl FiatTransaction {
    pub fn new_pending(data: &FiatQuoteUrlData, country: Option<String>, provider_transaction_id: Option<String>) -> Self {
        let quote = &data.quote;
        let now = Utc::now();

        Self {
            id: quote.id.clone(),
            asset_id: quote.asset.id.clone(),
            transaction_type: quote.quote_type.clone(),
            provider: quote.provider.id,
            provider_transaction_id,
            status: FiatTransactionStatus::Pending,
            country,
            fiat_amount: quote.fiat_amount,
            fiat_currency: quote.fiat_currency.clone(),
            value: quote.value.clone(),
            transaction_hash: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatTransactionUpdate {
    pub transaction_id: String,
    pub provider_transaction_id: Option<String>,
    pub status: FiatTransactionStatus,
    pub transaction_hash: Option<String>,
    pub fiat_amount: Option<f64>,
    pub fiat_currency: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct FiatTransactionData {
    pub transaction: FiatTransaction,
    pub details_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct FiatTransactionAssetData {
    pub transaction: FiatTransaction,
    pub asset: Asset,
    pub details_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum FiatTransactionStatus {
    Complete,
    Pending,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum FiatQuoteType {
    Buy,
    Sell,
}
