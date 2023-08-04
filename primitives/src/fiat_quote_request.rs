use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare()]
#[serde(rename_all = "camelCase")]
struct FiatBuyRequest {
    #[typeshare(skip)]
    ip_address: String, 
    fiat_currency: String,
    fiat_amount: f64,
    wallet_address: String,
}