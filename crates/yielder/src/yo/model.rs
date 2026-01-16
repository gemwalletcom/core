use alloy_primitives::U256;

/// Result from fetching position data via multicall
#[derive(Debug, Clone)]
pub struct PositionData {
    pub share_balance: U256,
    pub asset_balance: U256,
    pub latest_price: U256,
    pub latest_timestamp: u64,
    pub lookback_price: U256,
    pub lookback_timestamp: u64,
}
