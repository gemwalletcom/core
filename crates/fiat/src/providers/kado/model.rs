use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub data: T,
}

// #[derive(Debug, Deserialize, Clone)]
// #[serde(rename_all = "camelCase")]
// pub struct Assets {
//     pub assets: Vec<Asset>,
// }

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Blockchains {
    pub blockchains: Vec<Blockchain>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Blockchain {
    pub network: String,
    pub associated_assets: Vec<Asset>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub official_chain_id: String,
    pub symbol: String,
    pub address: Option<String>,
    pub ramp_products: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    #[serde(rename = "type")]
    pub webhook_type: String,
    pub data: WebhookData,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebhookData {
    pub id: String,
    pub tx_hash: String,
    pub currency_type: String,
    pub buy_amount: Amount,
    pub receive_amount: Amount,
    pub processing_fee: Amount,
    pub gas_fee: Amount,
    pub wallet_address: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteData {
    pub quote: Quote,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub receive_unit_count_after_fees: Amount,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Amount {
    pub amount: Option<f64>,
    pub unit: Option<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteQuery {
    pub transaction_type: String,
    pub blockchain: String,
    pub asset: String,
    pub amount: f64,
    pub currency: String,
}
