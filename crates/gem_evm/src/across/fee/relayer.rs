use num_bigint::BigInt;

pub struct CapitalCostConfig {
    pub lower_bound: BigInt,
    pub upper_bound: BigInt,
    pub cutoff: BigInt,
    pub decimals: u32,
}

pub struct RelayerFeeCalculator {}

impl RelayerFeeCalculator {
    /// Calculate the capital fee percent based on the configuration
    pub fn capital_fee_percent(amount_to_relay: BigInt, config: &CapitalCostConfig) -> BigInt {
        // Adjust the relayed amount to match the token's precision
        let adjusted_amount = amount_to_relay.clone() * BigInt::from(10u64.pow(config.decimals));

        // Calculate the fee using the lower bound
        let fee = (config.lower_bound.clone() * adjusted_amount.clone()) / BigInt::from(10u64.pow(config.decimals));

        // Ensure the fee does not exceed the upper bound
        if fee > config.upper_bound {
            config.upper_bound.clone()
        } else {
            fee
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capital_fee_percent() {
        let config = CapitalCostConfig {
            lower_bound: BigInt::from(100),  // Example: 0.0001
            upper_bound: BigInt::from(7500), // Example: 0.075
            cutoff: BigInt::from(300000),    // Example: 0.3
            decimals: 18,
        };

        let amount_to_relay = BigInt::from(1000000000000000000u64); // 1 WETH in wei

        // Calculate capital fee percent
        let capital_fee = RelayerFeeCalculator::capital_fee_percent(amount_to_relay.clone(), &config);
        println!("Capital Fee Percent: {:?}", capital_fee);
    }
}
