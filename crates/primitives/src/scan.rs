use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ScanAddress {
    pub name: Option<String>,
    pub address: String,
    pub is_verified: bool,
    pub is_fraudulent: bool,
    pub is_memo_required: bool,
}
