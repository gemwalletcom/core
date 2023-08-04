use typeshare::typeshare;
use serde::{Serialize, Deserialize};

use crate::chain::Chain;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
#[typeshare(swift = "Equatable, Codable, Hashable")]
struct AssetId {
    chain: Chain,
    #[serde(rename = "tokenId")]
    token_id: Option<String>,
}