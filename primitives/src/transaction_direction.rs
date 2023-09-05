use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum TransactionDirection {
    #[serde(rename = "self")]
    SelfTransfer,
    Outgoing,
    Incoming,
}
