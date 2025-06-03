use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::Int;

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaPrioritizationFee {
    pub prioritization_fee: Int,
}
