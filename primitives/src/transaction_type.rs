use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "camelCase")]
pub enum TransactionType {
    Transfer,
    Swap,
    TokenApproval,
}

impl TransactionType {
    pub fn to_string(&self) -> String {
        match self {
            TransactionType::Transfer => "transfer".to_string(),
            TransactionType::Swap => "swap".to_string(),
            TransactionType::TokenApproval => "tokenApproval".to_string(),
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "transfer" => Some(Self::Transfer),
            "swap" => Some(Self::Swap),
            "tokenApproval" => Some(Self::TokenApproval),
            _ => None,
        }
    }
}

impl Default for TransactionType {
    fn default() -> Self {
        Self::Transfer
    }
}
