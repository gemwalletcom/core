use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotToken {
    pub name: String,
    pub wei_decimals: i32,
    pub index: i32,
    pub token_id: String,
    pub sz_decimals: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotTokensResponse {
    pub tokens: Vec<SpotToken>,
}
