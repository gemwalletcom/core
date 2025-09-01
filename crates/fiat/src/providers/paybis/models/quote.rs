use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub amount: String,
    pub direction_change: String,
    pub is_received_amount: bool,
    pub currency_code_from: String,
    pub currency_code_to: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaybisQuote {
    pub currency_code_to: String,
    pub payment_methods: Vec<PaymentMethod>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethod {
    pub amount_to: AmountInfo,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmountInfo {
    pub amount: String,
}
