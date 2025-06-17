use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Status {
    pub blockbook: Blockbook,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Blockbook {
    #[serde(rename = "bestHeight")]
    pub best_height: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub page: i64,
    pub total_pages: i64,
    pub txs: Vec<Transaction>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddressTransactions {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub txid: String,
    pub value: String,
    pub value_in: String,
    pub fees: String,
    pub block_time: i64,
    pub block_height: i64,
    pub vin: Vec<Input>,
    pub vout: Vec<Output>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub is_address: bool,
    pub addresses: Option<Vec<String>>, // will be optional for Coinbase Input
    pub value: String,
    pub n: i64,
    pub tx_id: Option<String>, // will be optional for Coinbase Input
    pub vout: Option<i64>,     // will be optional for Coinbase Input
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub is_address: bool,
    pub addresses: Option<Vec<String>>,
    pub value: String,
    pub n: i64,
}
