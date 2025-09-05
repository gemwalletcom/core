use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatCurrency {
    pub symbol: String,
    pub name: String,
    pub payment_options: Vec<PaymentOption>,
    pub supporting_countries: Vec<String>,
    pub is_popular: bool,
    pub is_allowed: bool,
    pub round_off: u32,
    pub is_pay_out_allowed: bool,
    pub icon: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentOption {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub max_amount: Option<f64>,
    pub min_amount: Option<f64>,
    pub max_amount_for_pay_out: Option<f64>,
    pub min_amount_for_pay_out: Option<f64>,
    pub is_nft_allowed: Option<bool>,
    pub processing_time: Option<String>,
    pub limit_currency: Option<String>,
}