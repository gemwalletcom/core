use gem_hypercore::perpetual_calculator;

#[derive(Debug, Default, uniffi::Object)]
pub struct PerpetualCalculator;

#[uniffi::export]
impl PerpetualCalculator {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_minimum_usdc(&self, price: f64, sz_decimals: i32, leverage: u8) -> u64 {
        perpetual_calculator::PerpetualCalculator::calculate_minimum_usdc(price, sz_decimals, leverage)
    }
}
