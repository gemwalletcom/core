use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct TransactionSwapMetadata {
    pub from_asset: String,
    pub from_value: String,
    pub to_asset: String,
    pub to_value: String,
}

