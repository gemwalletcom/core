use serde::{Deserialize, Serialize};

use crate::Route;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SimulateSwapRequest {
    pub offer_address: String,
    pub units: String,
    pub ask_address: String,
    pub slippage_tolerance: String,
    pub referral_address: String,
    pub referral_fee_bps: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwapSimulation {
    pub offer_jetton_wallet: String,
    pub ask_jetton_wallet: String,
    pub router: Router,
    pub ask_units: String,
    pub min_ask_units: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Router {
    pub address: String,
    pub major_version: u8,
    pub minor_version: u8,
}

impl Router {
    pub(super) fn is_supported_v2(&self) -> bool {
        self.major_version == 2 && matches!(self.minor_version, 1 | 2)
    }
}

#[derive(Debug)]
pub(super) struct QuotePath {
    pub(super) to_value: String,
    pub(super) routes: Vec<Route>,
}
