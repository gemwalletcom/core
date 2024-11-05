use crate::fiat_provider::FiatProvider;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatQuote {
    pub provider: FiatProvider,
    #[serde(rename = "type")]
    pub quote_type: FiatQuoteType,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub crypto_amount: f64,
    pub redirect_url: String,
}

#[typeshare(swift = "Equatable, Sendable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FiatQuoteType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct FiatQuotes {
    pub quotes: Vec<FiatQuote>,
}
