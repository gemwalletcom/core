use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::typeshare::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct HypercoreCandlestick {
    pub t: UInt64, // Open time (timestamp in milliseconds)
    pub o: String, // Open price
    pub h: String, // High price
    pub l: String, // Low price
    pub c: String, // Close price
}
