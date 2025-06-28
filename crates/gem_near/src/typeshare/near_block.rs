use crate::typeshare::UInt64;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct NearBlock {
    pub header: NearBlockHeader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct NearBlockHeader {
    pub hash: String,
    pub height: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct NearGenesisConfig {
    pub chain_id: String,
}
