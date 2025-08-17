use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrpAccountResult {
    pub account_data: Option<XrpAccount>,
    pub ledger_current_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct XrpAccountObjects<T> {
    pub account_objects: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XrpAccountAsset {
    pub low_limit: XrpAssetLine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrpAssetLine {
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrpAccountLinesResult {
    pub lines: Option<Vec<XrpAccountLine>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrpAccountLine {
    pub account: String,
    pub balance: String,
    pub currency: String,
}
