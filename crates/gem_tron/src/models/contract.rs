use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronSmartContractCall {
    pub contract_address: String,
    pub function_selector: String,
    pub parameter: Option<String>,
    pub fee_limit: Option<u32>,
    pub call_value: Option<u32>,
    pub owner_address: String,
    pub visible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronSmartContractResult {
    pub result: TronSmartContractResultMessage,
    pub constant_result: Vec<String>,
    pub energy_used: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronSmartContractResultMessage {
    pub result: bool,
    pub message: Option<String>,
}
