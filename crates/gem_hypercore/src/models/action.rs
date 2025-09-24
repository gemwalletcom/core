use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreExchangeRequest {
    pub action: HypercoreExchangeAction,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypercoreExchangeAction {
    #[serde(rename = "type")]
    pub action_type: String,
}
