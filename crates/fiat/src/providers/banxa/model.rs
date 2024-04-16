use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct Coins {
    pub coins: Vec<Asset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Asset {
    pub coin_code: String,
    pub blockchains: Vec<Blockchain>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Blockchain {
    pub code: String,
    pub contract_id: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct OrderData<T> {
    pub order: T,
}

#[derive(Debug, Deserialize)]
pub struct Order {
    pub id: String,
    pub checkout_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderRequest {
    pub account_reference: String,
    pub source: String,
    pub source_amount: String,
    pub target: String,
    pub blockchain: String,
    pub wallet_address: String,
    pub return_url_on_success: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Prices {
    pub spot_price: String,
    pub prices: Vec<Price>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Price {
    pub network_fee: String,
    pub fee_amount: String,
    pub fiat_amount: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderDetails {
    pub id: String,
    pub coin_code: String,
    pub fiat_amount: f64,
    pub fiat_code: String,
    pub wallet_address: String,
    pub tx_hash: Option<String>,
    pub blockchain: Blockchain,
    pub fee: Option<f64>,
    pub merchant_fee: Option<f64>,
    pub network_fee: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Webhook {
    pub order_id: String,
    pub status: String,
}
