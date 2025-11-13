pub struct PerpetualCalculator;

impl Default for PerpetualCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl PerpetualCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Hyperliquid requires minimum $10 notional value (size × price).
    pub fn calculate_minimum_usdc(&self, price: f64, sz_decimals: i32, leverage: u8) -> u64 {
        let size_multiplier = 10_f64.powi(sz_decimals);
        let rounded_size = ((10.0 / price) * size_multiplier).ceil() / size_multiplier;
        let min_usdc = ((rounded_size * price / leverage as f64) * 100.0).ceil() / 100.0;

        (min_usdc * 1_000_000.0) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_minimum_usdc() {
        let calculator = PerpetualCalculator::new();

        assert_eq!(calculator.calculate_minimum_usdc(100_000.0, 5, 1), 10_000_000);
        assert_eq!(calculator.calculate_minimum_usdc(3_500.0, 4, 3), 3_390_000);
        assert_eq!(calculator.calculate_minimum_usdc(487.0, 2, 1), 14_610_000);
        assert_eq!(calculator.calculate_minimum_usdc(200.0, 1, 10), 2_000_000);
        assert_eq!(calculator.calculate_minimum_usdc(0.5, 0, 1), 10_000_000);
    }
}
