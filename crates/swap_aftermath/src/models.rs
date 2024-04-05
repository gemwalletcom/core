use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeQuote {
    pub coin_in_type: String,
    pub coin_out_type: String,
    pub coin_in_amount: String,
    pub external_fee: ExternalFee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalFee {
    pub recipient: String,
    pub fee_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub coin_in: CoinIn,
    pub coin_out: CoinOut,
    pub spot_price: f64,
    pub paths: Vec<Path>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeTx {
    pub wallet_address: String,
    pub complete_route: TradeQuoteResponse,
    pub slippage: f32,
    pub is_sponsored_tx: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeQuoteResponse {
    pub routes: Vec<Route>,
    pub spot_price: f64,
    pub coin_in: CoinIn,
    pub coin_out: CoinOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinIn {
    #[serde(rename = "type")]
    pub type_field: String,
    pub amount: String,
    pub trade_fee: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinOut {
    #[serde(rename = "type")]
    pub type_field: String,
    pub amount: String,
    pub trade_fee: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Path {
    pub coin_in: CoinIn,
    pub coin_out: CoinOut,
    pub spot_price: f64,
    pub protocol_name: String,
    pub pool: Value,
}
