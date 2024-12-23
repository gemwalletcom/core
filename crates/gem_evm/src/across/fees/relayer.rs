// https://github.com/across-protocol/sdk/blob/master/src/relayFeeCalculator/relayFeeCalculator.ts
use num_bigint::BigInt;
use num_traits::Zero;
use std::cmp::{max, min};

#[derive(Debug, Clone)]
pub struct CapitalCostConfig {
    pub lower_bound: BigInt,
    pub upper_bound: BigInt,
    pub cutoff: BigInt,
    pub decimals: u32,
}

pub struct RelayerFeeCalculator {
    fixed_point_adjustment: BigInt,
    max_big_int: BigInt,
}

impl Default for RelayerFeeCalculator {
    fn default() -> Self {
        Self {
            fixed_point_adjustment: BigInt::from(10u64.pow(18)),
            max_big_int: BigInt::from(i64::MAX), // Number.MAX_SAFE_INTEGER
        }
    }
}

impl RelayerFeeCalculator {
    /// Calculate the capital fee percent based on the configuration
    pub fn capital_fee_percent(&self, amount_to_relay: &BigInt, config: &CapitalCostConfig) -> BigInt {
        // If amount is 0, then the capital fee % should be the max 100%
        let zero = BigInt::zero();
        if amount_to_relay == &zero {
            return self.max_big_int.clone();
        }

        // Scale amount "y" to 18 decimals
        let scale_factor = BigInt::from(10).pow(18 - config.decimals);
        let y = amount_to_relay * &scale_factor;

        // At a minimum, the fee will be equal to lower bound * y
        let min_charge = &config.lower_bound * &y / &self.fixed_point_adjustment;

        // Special case: if cutoff is 0, return upper bound
        if config.cutoff == zero {
            return config.upper_bound.clone();
        }

        // Calculate triangle portion
        let y_triangle = min(&config.cutoff, &y);

        // triangleSlope is slope of fee curve from lower bound to upper bound
        let triangle_slope = if config.cutoff == zero {
            BigInt::from(0)
        } else {
            (&config.upper_bound - &config.lower_bound) * &self.fixed_point_adjustment / &config.cutoff
        };

        let triangle_height = &triangle_slope * y_triangle / &self.fixed_point_adjustment;
        let triangle_charge = &triangle_height * y_triangle / BigInt::from(2) / &self.fixed_point_adjustment;

        // For amounts above cutoff, apply the remainder charge
        let y_diff = &y - &config.cutoff;
        let y_remainder = max(&zero, &y_diff);
        let remainder_charge = y_remainder * (&config.upper_bound - &config.lower_bound) / &self.fixed_point_adjustment;

        // Calculate final fee percentage
        (min_charge + triangle_charge + remainder_charge) * &self.fixed_point_adjustment / &y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ether_conv::to_bn_wei;

    #[test]
    fn test_capital_fee_percent() {
        let calculator = RelayerFeeCalculator::default();

        // Setup ETH config
        let _eth_config = CapitalCostConfig {
            decimals: 18,
            lower_bound: to_bn_wei("0.0001", 18),
            upper_bound: to_bn_wei("0.000075", 18),
            cutoff: to_bn_wei("0.3", 18),
        };

        // Setup USDC config
        let _usdc_config = CapitalCostConfig {
            decimals: 6,
            lower_bound: to_bn_wei("0.0001", 18),
            upper_bound: BigInt::from(0), // FIXME shouldn't be zero?
            cutoff: BigInt::from(100000),
        };

        // Setup WBTC config
        let _wbtc_config = CapitalCostConfig {
            decimals: 8,
            lower_bound: to_bn_wei("0.0003", 18),
            upper_bound: to_bn_wei("0.002", 18),
            cutoff: to_bn_wei("15", 18),
        };

        // Setup DAI config
        let _dai_config = CapitalCostConfig {
            decimals: 18,
            lower_bound: to_bn_wei("0.0003", 18),
            upper_bound: to_bn_wei("0.0015", 18),
            cutoff: to_bn_wei("500000", 18),
        };

        // Setup ZERO_CUTOFF_DAI config
        let zero_cutoff_dai_config = CapitalCostConfig {
            decimals: 18,
            lower_bound: to_bn_wei("0.0003", 18),
            upper_bound: to_bn_wei("0.0015", 18),
            cutoff: BigInt::from(0),
        };

        // Test 1 ETH
        let fee = calculator.capital_fee_percent(&to_bn_wei("1", 18), &_eth_config);
        assert_eq!(fee, BigInt::from(78750000000001_u64));

        // FIXME Test 100 USDC
        // let fee = calculator.capital_fee_percent(&to_bn_wei("100", 6), &_usdc_config);
        // assert_eq!(fee, BigInt::from(127710000000000_u64));

        // Test near zero amount for WBTC
        let fee = calculator.capital_fee_percent(&to_bn_wei("0.001", 8), &_wbtc_config);
        assert_eq!(fee, to_bn_wei("0.000300056666666", 18));

        // Test near zero amount for DAI
        let fee = calculator.capital_fee_percent(&to_bn_wei("1", 18), &_dai_config);
        assert_eq!(fee, to_bn_wei("0.0003000012", 18));

        // Test amount below cutoff for WBTC
        let fee = calculator.capital_fee_percent(&to_bn_wei("14.999", 8), &_wbtc_config);
        assert_eq!(fee, to_bn_wei("0.00114994333333333", 18));

        // Test amount below cutoff for DAI
        let fee = calculator.capital_fee_percent(&to_bn_wei("499999", 18), &_dai_config);
        assert_eq!(fee, to_bn_wei("0.0008999988", 18));

        // Test amount much larger than cutoff for WBTC
        let fee = calculator.capital_fee_percent(&to_bn_wei("600", 8), &_wbtc_config);
        assert_eq!(fee, to_bn_wei("0.001978749999999999", 18));

        // Test amount much larger than cutoff for DAI
        let fee = calculator.capital_fee_percent(&to_bn_wei("20000000", 18), &_dai_config);
        assert_eq!(fee, to_bn_wei("0.001485", 18));

        // Test zero cutoff DAI (should always return upper bound)
        let fee = calculator.capital_fee_percent(&to_bn_wei("1", 18), &zero_cutoff_dai_config);
        assert_eq!(fee, to_bn_wei("0.0015", 18));
        let fee = calculator.capital_fee_percent(&to_bn_wei("499999", 18), &zero_cutoff_dai_config);
        assert_eq!(fee, to_bn_wei("0.0015", 18));
    }
}
