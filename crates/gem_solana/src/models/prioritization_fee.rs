use serde::{Deserialize, Serialize};

use super::Int;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaPrioritizationFee {
    pub prioritization_fee: Int,
}
