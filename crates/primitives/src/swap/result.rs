use super::SwapStatus;
use crate::Chain;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapResult {
    pub status: SwapStatus,
    pub from_chain: Chain,
    pub from_tx_hash: String,
    pub to_chain: Option<Chain>,
    pub to_tx_hash: Option<String>,
}
