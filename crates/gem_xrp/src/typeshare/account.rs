use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XrpAccountResult {
    pub account_data: Option<XrpAccount>,
    pub ledger_current_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XrpAccount {
    #[serde(rename = "Balance")]
    pub balance: String,
    #[serde(rename = "Sequence")]
    pub sequence: i32,
    #[serde(rename = "OwnerCount")]
    pub owner_count: i32,
    pub lines: Option<Vec<XrpAccountLine>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[typeshare(swift = "Sendable")]
pub struct XrpAccountObjects<T> {
    pub account_objects: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[typeshare(swift = "Sendable")]
pub struct XrpAccountAsset {
    pub low_limit: XrpAssetLine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XrpAssetLine {
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XrpAccountLinesResult {
    pub lines: Option<Vec<XrpAccountLine>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XrpAccountLine {
    pub account: String,
    pub balance: String,
    pub currency: String,
}
