use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::AssetId;
use crate::earn_provider::EarnProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct EarnPosition {
    pub asset_id: AssetId,
    pub provider: EarnProvider,
    pub name: String,
    pub vault_token_address: String,
    pub asset_token_address: String,
    pub vault_balance_value: BigInt,
    pub asset_balance_value: BigInt,
    pub balance: String,
    pub rewards: Option<String>,
    pub apy: Option<f64>,
}
