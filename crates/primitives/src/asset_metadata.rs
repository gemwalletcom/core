use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetMetaData {
    pub is_enabled: bool,
    pub is_balance_enabled: bool,
    pub is_buy_enabled: bool,
    pub is_sell_enabled: bool,
    pub is_swap_enabled: bool,
    pub is_stake_enabled: bool,
    pub staking_apr: Option<f64>,
    pub is_earn_enabled: bool,
    pub earn_apr: Option<f64>,
    pub is_pinned: bool,
    pub is_active: bool,
    pub rank_score: i32,
}
