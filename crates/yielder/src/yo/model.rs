use alloy_primitives::U256;
use gem_evm::u256::u256_to_f64;

const SECONDS_PER_YEAR: f64 = 365.25 * 24.0 * 60.0 * 60.0;

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

impl PositionData {
    pub fn calculate_apy(&self) -> Option<f64> {
        if self.lookback_price.is_zero() || self.lookback_timestamp >= self.latest_timestamp {
            return None;
        }

        let latest = u256_to_f64(self.latest_price);
        let lookback = u256_to_f64(self.lookback_price);
        let time_delta = (self.latest_timestamp - self.lookback_timestamp) as f64;

        if lookback == 0.0 || time_delta == 0.0 {
            return None;
        }

        let price_ratio = latest / lookback;
        let periods_per_year = SECONDS_PER_YEAR / time_delta;
        let apy = (price_ratio.powf(periods_per_year) - 1.0) * 100.0;

        if apy.is_finite() { Some(apy) } else { None }
    }
}
