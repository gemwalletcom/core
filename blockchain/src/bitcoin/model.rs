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
pub struct Transaction {
    pub txid: String,
}