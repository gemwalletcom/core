use num_bigint::BigInt;
use num_traits::Zero;
use std::collections::HashMap;

use primitives::AssetId;

/// Address -> Vec<BalanceDiff>
pub type BalanceDiffMap = HashMap<String, Vec<BalanceDiff>>;

pub struct BalanceDiff {
    pub asset_id: AssetId,
    pub from_value: BigInt,
    pub to_value: BigInt,
}

impl BalanceDiff {
    pub fn diff(&self) -> BigInt {
        self.to_value - self.from_value
    }
}
