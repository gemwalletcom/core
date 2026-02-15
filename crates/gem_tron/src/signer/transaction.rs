use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TronPayload {
    pub(crate) address: String,
    pub(crate) transaction: TronTransaction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TronTransaction {
    pub(crate) raw_data_hex: Option<String>,
    pub(crate) signature: Option<Vec<String>>,
    #[serde(flatten)]
    pub(crate) other: Map<String, Value>,
}
