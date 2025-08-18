use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiPay {
    pub tx_bytes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiPayRequest {
    pub sender_address: String,
    pub recipient_address: String,
    pub coins: Vec<String>,
    pub gas: Option<String>,
    pub amount: String,
    pub gas_budget: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiAddStakeRequest {
    pub sender_address: String,
    pub validator_address: String,
    pub coins: Vec<String>,
    pub amount: String,
    pub gas_budget: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiUnstakeRequest {
    pub sender_address: String,
    pub delegation_id: String,
    pub gas_budget: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiMoveCallRequest {
    pub sender_address: String,
    pub object_id: String,
    pub module: String,
    pub function: String,
    pub arguments: Vec<String>,
    pub gas_budget: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiSplitCoinRequest {
    pub sender_address: String,
    pub coin: String,
    pub split_amounts: Vec<String>,
    pub gas_budget: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiBatchRequest {
    pub sender_address: String,
    pub gas_budget: String,
}
