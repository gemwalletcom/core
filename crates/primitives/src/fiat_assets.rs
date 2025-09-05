use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::currency::Currency;
use crate::{AssetId, FiatQuoteType, PaymentType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatAssets {
    pub version: u32,
    pub asset_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatAssetLimits {
    pub currency: Currency,
    pub payment_type: PaymentType,
    pub quote_type: FiatQuoteType,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatAsset {
    pub id: String,
    pub asset_id: Option<AssetId>,
    pub provider: String,
    pub symbol: String,
    pub network: Option<String>,
    pub token_id: Option<String>,
    pub enabled: bool,
    pub unsupported_countries: HashMap<String, Vec<String>>,
    pub buy_limits: Vec<FiatAssetLimits>,
    pub sell_limits: Vec<FiatAssetLimits>,
}
