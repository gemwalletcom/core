use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPAccountResult {
    pub account_data: Option<XRPAccount>,
    pub ledger_current_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPAccount {
    #[serde(rename = "Balance")]
    pub balance: String,
    #[serde(rename = "Sequence")]
    pub sequence: i32,
    #[serde(rename = "OwnerCount")]
    pub owner_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[typeshare(swift = "Sendable")]
pub struct XRPAccountObjects<T> {
    pub account_objects: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[typeshare(swift = "Sendable")]
pub struct XRPAccountAsset {
    pub low_limit: XRPAssetLine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPAssetLine {
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPAccountLinesResult {
    pub lines: Option<Vec<XRPAccountLine>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPAccountLine {
    pub account: String,
    pub balance: String,
    pub currency: String,
}