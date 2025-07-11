use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct AlgorandVersions {
    #[serde(rename = "genesis-id")]
    pub genesis_id: String,
    #[serde(rename = "genesis-hash")]
    pub genesis_hash: String,
}
