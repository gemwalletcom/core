#[typeshare()]
#[serde(rename_all = "camelCase")]
struct FiatBuyRequest {
    #[serde(skip)]
    ip_address: String, 
    fiat_currency: String,
    fiat_amount: f64,
    wallet_address: String,
}