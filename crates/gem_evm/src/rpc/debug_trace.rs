use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PrestateDiffResult {
    pub pre: HashMap<String, AccountState>,
    pub post: HashMap<String, AccountState>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountState {
    pub balance: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallFrame {
    pub gas_used: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub revert_reason: Option<String>,
    #[serde(default)]
    pub logs: Vec<CallLog>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CallLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
}
