use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct PolkadotBlock {
    pub number: String,
    pub extrinsics: Vec<PolkadotExtrinsic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct PolkadotExtrinsic {
    pub hash: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct PolkadotNodeVersion {
    pub chain: String,
}