use std::collections::HashMap;

use primitives::{AssetId, BalanceDiff, SimulationResult};

use super::custom_types::GemBigInt;

pub type GemSimulationResult = SimulationResult;
pub type GemBalanceDiff = BalanceDiff;

#[uniffi::remote(Record)]
pub struct SimulationResult {
    pub success: bool,
    pub error: Option<String>,
    pub logs: Vec<String>,
    pub units_consumed: Option<u64>,
    pub balance_changes: HashMap<String, Vec<GemBalanceDiff>>,
}

#[uniffi::remote(Record)]
pub struct BalanceDiff {
    pub asset_id: AssetId,
    pub from_value: Option<GemBigInt>,
    pub to_value: Option<GemBigInt>,
    pub diff: GemBigInt,
}
