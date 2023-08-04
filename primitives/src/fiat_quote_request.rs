use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare()]
#[serde(rename_all = "camelCase")]
pub struct FiatBuyRequest {
    #[typeshare(skip)]
    pub ip_address: String, 
    pub fiat_currency: String,
    pub fiat_amount: f64,
    pub wallet_address: String,
}