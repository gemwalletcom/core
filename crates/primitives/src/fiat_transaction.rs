use crate::{AssetId, FiatProviderName, FiatQuoteUrlData};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatTransaction {
    pub asset_id: AssetId,
    pub transaction_type: FiatQuoteType,
    pub provider_id: FiatProviderName,
    pub provider_transaction_id: Option<String>,
    pub status: FiatTransactionStatus,
    #[typeshare(skip)]
    pub country: Option<String>,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub value: String,
    pub transaction_hash: Option<String>,
    pub address: Option<String>,
}

impl FiatTransaction {
    pub fn new_pending(data: &FiatQuoteUrlData, country: Option<String>, provider_transaction_id: Option<String>) -> Self {
        let quote = &data.quote;
        Self {
            asset_id: quote.asset.id.clone(),
            transaction_type: quote.quote_type.clone(),
            provider_id: quote.provider.id,
            provider_transaction_id,
            status: FiatTransactionStatus::Pending,
            country,
            fiat_amount: quote.fiat_amount,
            fiat_currency: quote.fiat_currency.clone(),
            value: quote.value.clone(),
            transaction_hash: None,
            address: Some(data.wallet_address.clone()),
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
    pub address: Option<String>,
    pub fiat_amount: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatTransactionInfo {
    pub transaction: FiatTransaction,
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
