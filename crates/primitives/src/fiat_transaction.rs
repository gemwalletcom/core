use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

use crate::AssetId;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatTransaction {
    pub asset_id: Option<AssetId>,
    pub provider_id: String,
    pub provider_transaction_id: String,
    pub status: FiatTransactionStatus,
    pub symbol: String,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub transaction_hash: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum FiatTransactionStatus {
    Complete,
    Pending,
    Failed,
    Unknown,
}
