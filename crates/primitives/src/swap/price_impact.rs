use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SwapPriceImpactType {
    Positive,
    Low,
    Medium,
    High,
}
