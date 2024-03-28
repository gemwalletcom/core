use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Quote {
    #[serde(rename = "CARD_PAYMENT")]
    pub card_payment: QuoteData,
    pub asset: QuoteAsset,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteData {
    //fiat_currency: String,
    pub crypto_amount: String,
    //fiat_value: u32,
    //base_ramp_fee: f64,
    //applied_fee: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteAsset {
    pub symbol: String,
    pub chain: String,
    pub decimals: u32,
    pub address: Option<String>,
    //enabled: bool,
    //hidden: bool,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QuoteAssets {
    pub assets: Vec<QuoteAsset>,
}

impl QuoteAsset {
    pub fn crypto_asset_symbol(&self) -> String {
        format!("{}_{}", self.chain, self.symbol)
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub crypto_asset_symbol: String,
    pub fiat_currency: String,
    pub fiat_value: f64,
}
