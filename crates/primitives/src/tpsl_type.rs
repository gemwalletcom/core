use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
pub enum TpslType {
    #[serde(rename = "tp")]
    TakeProfit,
    #[serde(rename = "sl")]
    StopLoss,
}
