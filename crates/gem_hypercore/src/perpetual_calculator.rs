// https://hyperliquid.gitbook.io/hyperliquid-docs/trading/contract-specifications
// https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/error-responses
const MIN_NOTIONAL_VALUE_USD: f64 = 10.0;
const USDC_CENTS_MULTIPLIER: f64 = 100.0;
const USDC_DECIMALS_MULTIPLIER: f64 = 1_000_000.0;

pub struct PerpetualCalculator;

impl PerpetualCalculator {
    
    /// Hyperliquid requires minimum $10 notional value (size × price).
    pub fn calculate_minimum_usdc(price: f64, sz_decimals: i32, leverage: u8) -> u64 {
        let size_multiplier = 10_f64.powi(sz_decimals);
        let rounded_size = ((MIN_NOTIONAL_VALUE_USD / price) * size_multiplier).ceil() / size_multiplier;
        let min_usdc = ((rounded_size * price / leverage as f64) * USDC_CENTS_MULTIPLIER).ceil() / USDC_CENTS_MULTIPLIER;

        (min_usdc * USDC_DECIMALS_MULTIPLIER) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_minimum_usdc() {
        assert_eq!(PerpetualCalculator::calculate_minimum_usdc(100_000.0, 5, 1), 10_000_000);
        assert_eq!(PerpetualCalculator::calculate_minimum_usdc(3_500.0, 4, 3), 3_390_000);
        assert_eq!(PerpetualCalculator::calculate_minimum_usdc(487.0, 2, 1), 14_610_000);
        assert_eq!(PerpetualCalculator::calculate_minimum_usdc(200.0, 1, 10), 2_000_000);
        assert_eq!(PerpetualCalculator::calculate_minimum_usdc(0.5, 0, 1), 10_000_000);
    }
}
