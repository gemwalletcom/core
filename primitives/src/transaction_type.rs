use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Transfer,
}

impl TransactionType {
    pub fn to_string(&self) -> String {
        match self {
            TransactionType::Transfer => "transfer".to_string(),
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "transfer" => Some(Self::Transfer),
            _ => None,
        }
    }
}

impl Default for TransactionType {
    fn default() -> Self {
        Self::Transfer
    }
}
