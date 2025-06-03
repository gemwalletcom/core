use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaPrioritizationFee {
    #[serde(rename = "prioritizationFee")]
    pub prioritization_fee: i32,
}
