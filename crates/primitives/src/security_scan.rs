use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SecurityResponse {
    pub malicious: bool,
    pub reason: String,
    pub provider: String,
    pub metadata: Option<SecurityMetadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SecurityMetadata {
    pub name: String,
    pub verified: bool,
    pub required_memo: bool,
}
