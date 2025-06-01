use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_serializers::deserialize_biguint_from_str;

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
    #[serde(rename = "computationCost", deserialize_with = "deserialize_biguint_from_str")]
    pub computation_cost: BigUint,
    #[serde(rename = "storageCost", deserialize_with = "deserialize_biguint_from_str")]
    pub storage_cost: BigUint,
    #[serde(rename = "storageRebate", deserialize_with = "deserialize_biguint_from_str")]
    pub storage_rebate: BigUint,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinMetadata {
    pub id: String,
    pub name: String,
    pub decimals: i32,
    pub symbol: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub coin_type: String,
    pub total_balance: String,
}
