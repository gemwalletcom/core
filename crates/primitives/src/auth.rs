use crate::Chain;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AuthNonce {
    pub nonce: String,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AuthMessage {
    pub chain: Chain,
    pub address: String,
    pub auth_nonce: AuthNonce,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AuthPayload {
    pub device_id: String,
    pub chain: Chain,
    pub address: String,
    pub nonce: String,
    pub signature: String,
}
