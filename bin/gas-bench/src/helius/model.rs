use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct HeliusPriorityFeeRequest {
    pub jsonrpc: &'static str,
    pub id: &'static str,
    pub method: &'static str,
    pub params: Vec<HeliusPriorityFeeParams>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeliusPriorityFeeParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_keys: Option<Vec<String>>,
    pub options: HeliusPriorityFeeOptions,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeliusPriorityFeeOptions {
    pub include_all_priority_fee_levels: bool,
    pub lookback_slots: u32,
}

#[derive(Debug, Deserialize)]
pub struct HeliusPriorityFeeResponse {
    pub result: HeliusPriorityFeeResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeliusPriorityFeeResult {
    pub priority_fee_levels: Option<HeliusPriorityFeeLevels>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeliusPriorityFeeLevels {
    pub low: f64,
    pub medium: f64,
    pub high: f64,
}

#[derive(Debug)]
pub struct HeliusPriorityFees {
    pub low: u64,
    pub medium: u64,
    pub high: u64,
}

impl HeliusPriorityFees {
    pub fn from_levels(levels: &HeliusPriorityFeeLevels) -> Self {
        Self {
            low: levels.low as u64,
            medium: levels.medium as u64,
            high: levels.high as u64,
        }
    }
}
