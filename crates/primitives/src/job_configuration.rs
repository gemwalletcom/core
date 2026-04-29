use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct JobConfiguration {
    pub initial_interval_ms: u32,
    pub max_interval_ms: u32,
    pub step_factor: f32,
}
