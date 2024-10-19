use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultData<T> {
    pub data: T,
}

pub type Digests = ResultData<Vec<Digest>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Digest {
    pub digest: String,
    pub effects: Effect,
    #[serde(rename = "balanceChanges")]
    pub balance_changes: Option<Vec<BalanceChange>>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    pub owner: Owner,
    #[serde(rename = "coinType")]
    pub coin_type: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Owner {
    String(String),
    OwnerObject(OwnerObject),
}

impl Owner {
    pub fn get_address_owner(&self) -> Option<String> {
        match self {
            Owner::String(_) => None,
            Owner::OwnerObject(obj) => obj.address_owner.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerObject {
    #[serde(rename = "AddressOwner")]
    pub address_owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    #[serde(rename = "gasUsed")]
    pub gas_used: GasUsed,
    pub status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasUsed {
    #[serde(rename = "computationCost")]
    pub computation_cost: String,
    #[serde(rename = "storageCost")]
    pub storage_cost: String,
    #[serde(rename = "storageRebate")]
    pub storage_rebate: String,
    #[serde(rename = "nonRefundableStorageFee")]
    pub non_refundable_storage_fee: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(rename = "parsedJson")]
    pub parsed_json: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStake {
    pub amount: String,
    pub staker_address: String,
    pub validator_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventUnstake {
    pub principal_amount: String,
    pub reward_amount: String,
    pub staker_address: String,
    pub validator_address: String,
}
