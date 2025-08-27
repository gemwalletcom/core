use crate::Chain;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub chain: Chain,
    pub address: String,
    pub derivation_path: String,
    pub extended_public_key: Option<String>,
}
