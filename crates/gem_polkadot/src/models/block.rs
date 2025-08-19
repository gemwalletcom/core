use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotBlock {
    pub number: String,
    pub extrinsics: Vec<PolkadotExtrinsic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotExtrinsic {
    pub hash: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotNodeVersion {
    pub chain: String,
}
