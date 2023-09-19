use serde::{Serialize, Deserialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    pub owner: Owner,
    #[serde(rename = "coinType")]
    pub coin_type: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    #[serde(rename = "AddressOwner")]
    pub address_owner: Option<String>,
    #[serde(rename = "ObjectOwner")]
    pub object_owner: Option<String>,
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