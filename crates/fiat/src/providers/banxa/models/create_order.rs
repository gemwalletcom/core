use serde::{Deserialize, Serialize};

pub const PAYMENT_METHOD_CARD: &str = "debit-credit-card";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    pub payment_method_id: String,
    pub crypto: String,
    pub blockchain: String,
    pub fiat: String,
    pub fiat_amount: String,
    pub wallet_address: String,
    pub redirect_url: String,
    pub external_customer_id: String,
    pub external_order_id: String,
}

impl CreateOrderRequest {
    pub fn new(external_order_id: String, crypto: String, fiat: String, fiat_amount: f64, blockchain: String, wallet_address: String, redirect_url: String) -> Self {
        Self {
            payment_method_id: PAYMENT_METHOD_CARD.to_string(),
            crypto,
            blockchain,
            fiat,
            fiat_amount: fiat_amount.to_string(),
            external_customer_id: wallet_address.clone(),
            wallet_address,
            redirect_url,
            external_order_id,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutOrder {
    pub id: String,
    pub checkout_url: String,
}
