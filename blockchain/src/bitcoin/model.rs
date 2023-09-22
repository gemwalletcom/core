use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Status {
    pub blockbook: Blockbook,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Blockbook {
    #[serde(rename = "bestHeight")]
    pub best_height: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Block {
    pub txs: Vec<Transaction>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub txid: String,
    pub value: String,
    pub value_in: String,
    pub fees: String,
    pub block_time: i64,
    pub block_height: i64,
    pub vin: Vec<Input>,
    pub vout: Vec<Input>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub is_address: bool,
    pub addresses: Option<Vec<String>>,
    pub value: String,
}