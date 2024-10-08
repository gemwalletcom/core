use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum TransactionDirection {
    #[serde(rename = "self")]
    SelfTransfer,
    Outgoing,
    Incoming,
}
