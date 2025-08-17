use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearBlock {
    pub header: NearBlockHeader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearBlockHeader {
    pub hash: String,
    pub height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearGenesisConfig {
    pub chain_id: String,
}
