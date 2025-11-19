use crate::{AssetId, FiatQuoteType, fiat_provider::FiatProvider};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct FiatQuote {
    pub provider: FiatProvider,
    #[serde(rename = "type")]
    pub quote_type: FiatQuoteType,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub crypto_amount: f64,
    pub crypto_value: String,
    pub redirect_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct FiatQuoteData {
    pub id: String,
    pub provider: FiatProvider,
    #[serde(rename = "type")]
    pub quote_type: FiatQuoteType,
    pub asset_id: AssetId,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub crypto_amount: f64,
}

impl FiatQuoteData {
    pub fn new(
        id: String,
        provider: FiatProvider,
        quote_type: FiatQuoteType,
        asset_id: AssetId,
        fiat_amount: f64,
        fiat_currency: String,
        crypto_amount: f64,
    ) -> Self {
        Self {
            id,
            provider,
            quote_type,
            asset_id,
            fiat_amount,
            fiat_currency,
            crypto_amount,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct FiatQuotesData {
    pub quotes: Vec<FiatQuoteData>,
    #[typeshare(skip)]
    pub errors: Vec<FiatQuoteError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatQuoteUrlRequest {
    pub quote_id: String,
    pub wallet_address: String,
    pub ip_address: String,
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatQuoteUrl {
    pub redirect_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct FiatQuotes {
    pub quotes: Vec<FiatQuote>,
    #[typeshare(skip)]
    pub errors: Vec<FiatQuoteError>,
}

impl FiatQuotes {
    pub fn new_error(error: FiatQuoteError) -> Self {
        Self {
            quotes: vec![],
            errors: vec![error],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct FiatQuoteError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub error: String,
}

impl FiatQuoteError {
    pub fn new(provider: Option<String>, error: String) -> Self {
        Self { provider, error }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatQuoteDataRequest {
    #[serde(rename = "type")]
    #[typeshare(skip)]
    pub quote_type: FiatQuoteType,
    pub fiat_currency: String,
    pub fiat_amount: f64,
    #[typeshare(skip)]
    pub ip_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatQuoteDataResponse {
    pub quote_id: String,
    pub fiat_amount: f64,
    pub crypto_amount: f64,
}

impl FiatQuoteDataResponse {
    pub fn new(quote_id: String, fiat_amount: f64, crypto_amount: f64) -> Self {
        Self {
            quote_id,
            fiat_amount,
            crypto_amount,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatAssetSymbol {
    pub symbol: String,
    pub network: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FiatQuoteUrlData {
    pub quote: FiatQuoteData,
    pub asset_symbol: FiatAssetSymbol,
    pub wallet_address: String,
    pub ip_address: String,
}
