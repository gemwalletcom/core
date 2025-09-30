use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotToken {
    pub name: String,
    pub wei_decimals: i32,
    pub index: i32,
    pub token_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotTokensResponse {
    pub tokens: Vec<SpotToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpotMetadataResponse(pub (SpotTokensResponse, serde_json::Value));

impl SpotMetadataResponse {
    pub fn token_data(&self) -> &SpotTokensResponse {
        &self.0.0
    }
}
