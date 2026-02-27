use std::collections::HashMap;

use num_bigint::BigInt;

use crate::AssetId;

/// Address -> Vec<BalanceDiff>
pub type BalanceDiffMap = HashMap<String, Vec<BalanceDiff>>;

#[derive(Debug, Clone)]
pub struct BalanceDiff {
    pub asset_id: AssetId,
    pub from_value: Option<BigInt>,
    pub to_value: Option<BigInt>,
    pub diff: BigInt,
}
