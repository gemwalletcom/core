use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbiosisTransactionData {
    pub chain_id: u64,
    pub data: String,
    pub to: String,
    pub value: Option<String>,
    pub function_selector: String,
    pub fee_limit: Option<u64>,
    pub from: Option<String>,
}
