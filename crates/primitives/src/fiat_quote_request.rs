use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatQuoteRequest {
    pub asset_id: String,
    #[typeshare(skip)]
    pub ip_address: String,
    pub fiat_currency: String,
    pub fiat_amount: f64,
    pub crypto_amount: Option<f64>,
    pub wallet_address: String,
    #[typeshare(skip)]
    pub provider_id: Option<String>,
}
