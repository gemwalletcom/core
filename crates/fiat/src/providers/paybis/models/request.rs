use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub crypto_wallet_address: String,
    pub currency_code_from: String,
    pub currency_code_to: String,
    pub requested_amount: f64,
    pub requested_amount_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_ip: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestResponse {
    pub request_id: String,
}

impl Request {
    pub fn new_buy(wallet_address: String, fiat_currency: String, crypto_currency: String, fiat_amount: f64, user_ip: Option<String>) -> Self {
        Self {
            crypto_wallet_address: wallet_address,
            currency_code_from: fiat_currency,
            currency_code_to: crypto_currency,
            requested_amount: fiat_amount,
            requested_amount_type: "from".to_string(),
            user_ip,
        }
    }

    pub fn new_sell(wallet_address: String, crypto_currency: String, fiat_currency: String, crypto_amount: f64, user_ip: Option<String>) -> Self {
        Self {
            crypto_wallet_address: wallet_address,
            currency_code_from: crypto_currency,
            currency_code_to: fiat_currency,
            requested_amount: crypto_amount,
            requested_amount_type: "from".to_string(),
            user_ip,
        }
    }
}
