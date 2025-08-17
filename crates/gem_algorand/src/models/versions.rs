use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorandVersions {
    #[serde(rename = "genesis-id")]
    pub genesis_id: String,
    #[serde(rename = "genesis-hash")]
    pub genesis_hash: String,
}
