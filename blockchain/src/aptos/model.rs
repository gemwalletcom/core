use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub block_height: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub block_height: String,
    pub transactions: Vec<Transaction>,
    // #[serde(rename = "type")]
    // pub transaction_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub sender: Option<String>,
    pub success: bool,
    pub gas_used: Option<String>,
    pub gas_unit_price: Option<String>,
    pub events: Option<Vec<Event>>,
    //pub payload: Option<Payload>,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub sequence_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub guid: Guid,
    pub data: Option<AmountData>,
    #[serde(rename = "type")]
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountData {
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guid {
    pub account_address: String,
}