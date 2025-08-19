use serde::{Deserialize, Serialize};

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
    pub gas_used: SuiGasUsed,
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
pub struct SuiGasUsed {
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
#[serde(rename_all = "camelCase")]
pub struct SuiBroadcastTransaction {
    pub digest: String,
}
