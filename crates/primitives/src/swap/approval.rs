use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
}
