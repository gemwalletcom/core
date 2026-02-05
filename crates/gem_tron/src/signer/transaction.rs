use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TronPayload {
    pub(crate) address: String,
    pub(crate) transaction: TronTransaction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) signature: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TronTransaction {
    pub(crate) raw_data: Option<TronRawData>,
    pub(crate) raw_data_hex: Option<String>,
    #[serde(default)]
    pub(crate) signature: Vec<String>,
    #[serde(rename = "txID")]
    pub(crate) tx_id: Option<String>,
    pub(crate) visible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TronRawData {
    pub(crate) contract: Vec<TronContract>,
    pub(crate) expiration: Option<u64>,
    pub(crate) fee_limit: Option<u64>,
    pub(crate) ref_block_bytes: Option<String>,
    pub(crate) ref_block_hash: Option<String>,
    pub(crate) timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TronContract {
    pub(crate) parameter: TronContractParameter,
    #[serde(rename = "type")]
    pub(crate) contract_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TronContractParameter {
    pub(crate) type_url: String,
    pub(crate) value: TronContractValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TronContractValue {
    pub(crate) contract_address: Option<String>,
    pub(crate) data: Option<String>,
    pub(crate) owner_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) call_value: Option<u64>,
}
