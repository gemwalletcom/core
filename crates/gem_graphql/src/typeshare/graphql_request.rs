use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct GraphqlRequest {
    pub operation_name: String,
    pub variables: HashMap<String, String>,
    pub query: String,
}