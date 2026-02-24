use super::SwapStatus;
use crate::TransactionSwapMetadata;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapResult {
    pub status: SwapStatus,
    pub metadata: Option<TransactionSwapMetadata>,
}
