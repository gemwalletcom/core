use serde::Deserialize;
use serde_serializers::{deserialize_f64_from_str, deserialize_option_f64_from_str};
use super::asset::Coin;

pub const ORDER_TYPE_BUY: &str = "buy";
pub const ORDER_TYPE_SELL: &str = "sell";

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub id: String,
    pub status: String,
    pub crypto: Coin,
    pub fiat: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fiat_amount: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub crypto_amount: f64,
    pub wallet_address: String,
    pub tx_hash: Option<String>,
    #[serde(deserialize_with = "deserialize_option_f64_from_str")]
    pub processing_fee: Option<f64>,
    #[serde(deserialize_with = "deserialize_option_f64_from_str")]
    pub network_fee: Option<f64>,
    pub order_type: String,
    pub country: Option<String>,
}