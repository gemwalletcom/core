use serde::{Deserialize, Serialize};

pub const WITHDRAW_EVENT: &str = "0x1::coin::WithdrawEvent";
pub const DEPOSIT_EVENT: &str = "0x1::coin::DepositEvent";

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
    pub data: Option<serde_json::Value>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Empty {}

impl Event {
    pub fn get_amount(&self) -> Option<String> {
        let data = self.data.clone()?;
        match self.event_type.as_str() {
            WITHDRAW_EVENT | DEPOSIT_EVENT => serde_json::from_value::<AmountData>(data).ok()?.amount,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDataCoinInfo {
    pub decimals: i32,
    pub name: String,
    pub symbol: String,
}
