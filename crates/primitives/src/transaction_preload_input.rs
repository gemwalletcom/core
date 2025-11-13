use crate::{TransactionInputType, TransactionType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionPreloadInput {
    pub input_type: TransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
}

impl TransactionPreloadInput {
    pub fn scan_type(&self) -> Option<TransactionType> {
        match &self.input_type {
            TransactionInputType::Transfer(_) => Some(TransactionType::Transfer),
            _ => None,
        }
    }

    pub fn get_website(&self) -> Option<String> {
        match &self.input_type {
            TransactionInputType::Generic(_, app_metadata, _) => Some(app_metadata.url.clone()),
            _ => None,
        }
    }
}
