use crate::AssetId;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatTransaction {
    pub asset_id: Option<AssetId>,
    pub transaction_type: FiatTransactionType,
    pub provider_id: String,
    pub provider_transaction_id: String,
    pub status: FiatTransactionStatus,
    pub symbol: String,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub transaction_hash: Option<String>,
    pub address: Option<String>,
    pub fee_provider: Option<f64>,
    pub fee_network: Option<f64>,
    pub fee_partner: Option<f64>,
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

#[typeshare(swift = "Equatable, Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum FiatTransactionType {
    Buy,
    Sell,
}
