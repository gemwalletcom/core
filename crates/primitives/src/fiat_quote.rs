use crate::{fiat_provider::FiatProvider, FiatTransactionType};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatQuote {
    pub provider: FiatProvider,
    #[serde(rename = "type")]
    pub quote_type: FiatTransactionType,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub crypto_amount: f64,
    pub redirect_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct FiatQuotes {
    pub quotes: Vec<FiatQuote>,
}
