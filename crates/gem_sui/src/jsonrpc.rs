use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Not};

pub enum SuiRpc {
    GetObject(String),
    GetMultipleObjects(Vec<String>),
    InspectTransactionBlock(String),
    NormalizedMoveFunction(Vec<String>),
}

impl Display for SuiRpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GetObject(_) => write!(f, "sui_getObject"),
            Self::GetMultipleObjects(_) => write!(f, "sui_multiGetObjects"),
            Self::InspectTransactionBlock(_) => write!(f, "sui_devInspectTransactionBlock"),
            Self::NormalizedMoveFunction(_) => write!(f, "sui_getNormalizedMoveFunction"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectDataOptions {
    #[serde(skip_serializing_if = "Not::not")]
    pub show_type: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub show_owner: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub show_display: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub show_content: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub show_bcs: bool,
}
