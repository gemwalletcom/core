use crate::{AssetId, FiatProviderName};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatTransaction {
    pub asset_id: Option<AssetId>,
    pub transaction_type: FiatQuoteType,
    pub provider_id: FiatProviderName,
    pub provider_transaction_id: String,
    pub status: FiatTransactionStatus,
    #[typeshare(skip)]
    pub country: Option<String>,
    #[typeshare(skip)]
    pub symbol: String,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub transaction_hash: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatTransactionInfo {
    pub transaction: FiatTransaction,
    pub details_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum FiatTransactionStatus {
    Complete,
    Pending,
    Failed,
    Unknown(String),
}

#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum FiatQuoteType {
    Buy,
    Sell,
}
