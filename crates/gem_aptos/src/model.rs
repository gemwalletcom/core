use serde::{Deserialize, Serialize};

use super::constants::{STAKE_DEPOSIT_EVENT, STAKE_WITHDRAW_EVENT};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub block_height: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub block_height: String,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub sender: Option<String>,
    pub success: bool,
    pub gas_used: Option<String>,
    pub gas_unit_price: Option<String>,
    pub events: Option<Vec<Event>>,
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
            STAKE_WITHDRAW_EVENT | STAKE_DEPOSIT_EVENT => serde_json::from_value::<AmountData>(data).ok()?.amount,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPayload {
    pub function: String,
    pub type_arguments: Vec<String>,
    pub arguments: Vec<String>,
    #[serde(rename = "type")]
    pub payload_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource<T> {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResourceData {
    CoinStore(CoinStore),
    CoinInfo(CoinInfo),
    Other(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinStore {
    pub coin: Coin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinInfo {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
}
