use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct ScanAddress {
    pub name: Option<String>,
    pub address: String,
    pub is_verified: bool,
    pub is_fradulent: bool,
    pub is_memo_required: bool,
}