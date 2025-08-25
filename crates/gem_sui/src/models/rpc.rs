#[cfg(feature = "rpc")]
use num_bigint::{BigInt, BigUint};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(feature = "rpc")]
use serde_serializers::deserialize_u64_from_str;
#[cfg(feature = "rpc")]
use serde_serializers::{deserialize_bigint_from_str, deserialize_biguint_from_str};

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBlocks {
    pub data: Vec<Digest>,
}

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checkpoint {
    pub epoch: String,
    pub sequence_number: String,
    pub digest: String,
    pub network_total_transactions: String,
    pub previous_digest: String,
    pub timestamp_ms: String,
    pub transactions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultData<T> {
    pub data: T,
}

#[cfg(feature = "rpc")]
pub type Digests = ResultData<Vec<Digest>>;

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Digest {
    pub digest: String,
    pub effects: Effect,
    #[serde(rename = "balanceChanges")]
    pub balance_changes: Option<Vec<BalanceChange>>,
    pub events: Vec<Event>,
    #[serde(rename = "timestampMs")]
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub timestamp_ms: u64,
}

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    pub owner: Owner,
    #[serde(rename = "coinType")]
    pub coin_type: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub amount: BigInt,
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

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    #[serde(rename = "gasUsed")]
    pub gas_used: GasUsed,
    pub status: Status,
    #[serde(rename = "gasObject")]
    pub gas_object: GasObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasObject {
    pub owner: Owner,
}

#[cfg(feature = "rpc")]
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
    #[serde(rename = "packageId")]
    pub package_id: String,
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

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub coin_type: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub total_balance: BigInt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSet {
    pub apys: Vec<ValidatorApy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorApy {
    pub address: String,
    pub apy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiSystemState {
    pub active_validators: Vec<ValidatorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorInfo {
    pub sui_address: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBroadcast {
    pub digest: String,
}
