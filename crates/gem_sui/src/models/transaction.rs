use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_serializers::{deserialize_biguint_from_str, deserialize_u64_from_str};

#[cfg(feature = "rpc")]
use super::account::GasObject;
#[cfg(feature = "rpc")]
use super::coin::BalanceChange;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiTransaction {
    pub effects: SuiEffects,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiStatus {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiEffects {
    pub gas_used: GasUsed,
    pub status: SuiStatus,
    pub created: Option<Vec<SuiObjectChange>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiObjectChange {
    pub reference: SuiObjectReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiObjectReference {
    pub object_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GasUsed {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub computation_cost: BigUint,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub storage_cost: BigUint,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub storage_rebate: BigUint,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub non_refundable_storage_fee: BigUint,
}

pub use TransactionBroadcast as SuiBroadcastTransaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionQueryResult {
    pub data: Vec<SuiTransaction>,
    pub has_next_page: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBroadcast {
    pub digest: String,
}

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBlocks {
    pub data: Vec<Digest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultData<T> {
    pub data: T,
}

#[cfg(feature = "rpc")]
pub type Digests = ResultData<Vec<Digest>>;

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
pub struct Effect {
    #[serde(rename = "gasUsed")]
    pub gas_used: GasUsed,
    pub status: Status,
    #[serde(rename = "gasObject")]
    pub gas_object: GasObject,
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
