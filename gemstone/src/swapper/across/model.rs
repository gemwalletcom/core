use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaggestedFeesRequest {
    pub input_token: String,
    pub output_token: String,
    pub amount: String,
    pub origin_chain_id: String,
    pub destination_chain_id: String,
    pub recipient: Option<String>,
    pub message: Option<String>,
    pub relayer: Option<String>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaggestedFeesResponse {
    pub estimated_fill_time_sec: u64,
    pub capital_fee_pct: String,
    pub capital_fee_total: String,
    pub is_amount_too_low: bool,
    pub quote_block: String,
    pub destination_spoke_pool_address: String,
    pub timestamp: String,
    pub spoke_pool_address: String,
    pub total_relay_fee: FeeStruct,
    pub relayer_capital_fee: FeeStruct,
    pub relayer_gas_fee: FeeStruct,
    pub lp_fee: FeeStruct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeStruct {
    pub pct: String,
    pub total: String,
}
