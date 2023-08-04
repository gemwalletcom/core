use typeshare::typeshare;
use serde::{Serialize, Deserialize};
use crate::fiat_provider::FiatProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable")]
#[serde(rename_all = "camelCase")]
pub struct FiatQuote {
    pub provider: FiatProvider,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub crypto_amount: f64,
    pub redirect_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare()]
pub struct FiatQuotes {
    pub quotes: Vec<FiatQuote>
}