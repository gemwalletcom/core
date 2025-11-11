use crate::{FiatQuoteType, fiat_provider::FiatProvider};
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
