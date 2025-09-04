use std::cmp::min;

use primitives::{fee::FeePriority, PriorityFeeValue, EVMChain};

use crate::models::fee::EthereumFeeHistory;

use num_bigint::BigInt;
use serde_serializers::bigint_from_hex_str;

pub fn get_fee_history_blocks(chain: EVMChain) -> u64 {
    let block_time = chain.to_chain().block_time();
    min(60 * 1000 / block_time, 15) as u64
}

pub fn get_reward_percentiles() -> [u64; 3] {
    [20, 40, 60]
}

pub struct FeeCalculator;

impl Default for FeeCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl FeeCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_min_priority_fee(
        &self,
        gas_used_ratios: &[f64],
        base_fee: &BigInt,
        default_min_priority_fee: u64,
    ) -> Result<u64, Box<dyn std::error::Error + Sync + Send>> {
        if gas_used_ratios.is_empty() || base_fee == &BigInt::from(0) {
            return Ok(default_min_priority_fee);
        }
        let avg_ratio: f64 = gas_used_ratios.iter().sum::<f64>() / gas_used_ratios.len() as f64;

        let result = match avg_ratio {
            r if r >= 0.9 => default_min_priority_fee,
            r if r >= 0.7 => default_min_priority_fee / 2,
            _ => default_min_priority_fee / 10,
        };
        Ok(result)
    }

    pub fn calculate_priority_fees(
        &self,
        fee_history: &EthereumFeeHistory,
        priorities: &[FeePriority],
        min_priority_fee: BigInt,
    ) -> Result<Vec<PriorityFeeValue>, Box<dyn std::error::Error + Sync + Send>> {
        if fee_history.reward.is_empty() {
            return Err("fee_history.reward is empty".into());
        }

        if priorities.len() != fee_history.reward[0].len() {
            return Err("priorities.len() != fee_history.reward[0].len()".into());
        }

        let rewards = &fee_history.reward;

        let mut columns: Vec<Vec<BigInt>> = vec![Vec::new(); priorities.len()];
        for row in rewards {
            for (i, hex_fee) in row.iter().enumerate().take(priorities.len()) {
                if let Ok(bn) = bigint_from_hex_str(hex_fee) {
                    columns[i].push(bn);
                }
            }
        }

        let result = priorities
            .iter()
            .zip(columns.iter())
            .map(|(&priority, fees)| {
                let value = if fees.is_empty() {
                    min_priority_fee.clone()
                } else {
                    let sum = fees.iter().cloned().fold(BigInt::from(0), |a, b| a + b);
                    let avg = &sum / BigInt::from(fees.len());
                    let min_value = min_priority_fee.clone();
                    if avg < min_value {
                        min_value
                    } else {
                        avg
                    }
                };

                PriorityFeeValue { priority, value }
            })
            .collect();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::fee::FeePriority;

    fn create_test_fee_history() -> EthereumFeeHistory {
        EthereumFeeHistory {
            reward: vec![
                vec!["0x54e0840".to_string(), "0x31e7fe5d".to_string(), "0x3b9aca04".to_string()],
                vec!["0x4b571c0".to_string(), "0x18bf8474".to_string(), "0x3b9aca00".to_string()],
                vec!["0x18e20bb9".to_string(), "0x32324960".to_string(), "0x3b9aca00".to_string()],
                vec!["0x38444c0".to_string(), "0x7bf60c0".to_string(), "0x31e7fe5d".to_string()],
                vec!["0x5f5e100".to_string(), "0x29b92700".to_string(), "0x39fbe24e".to_string()],
            ],
            base_fee_per_gas: vec![
                BigInt::from(2618877110u64),
                BigInt::from(2600645117u64),
                BigInt::from(2474034920u64),
                BigInt::from(2495024366u64),
                BigInt::from(2624620404u64),
                BigInt::from(2541053471u64),
            ],
            gas_used_ratio: vec![0.4787648265769147, 0.30434244444444447, 0.5349411706429458, 0.707018, 0.37411107986145914],
            oldest_block: 22832041,
        }
    }

    #[test]
    fn test_get_fee_history_blocks() {
        assert!(get_fee_history_blocks(primitives::EVMChain::Ethereum) > 0);
        assert!(get_fee_history_blocks(primitives::EVMChain::Arbitrum) > 0);
    }

    #[test]
    fn test_get_reward_percentiles() {
        assert_eq!(get_reward_percentiles(), [20, 40, 60]);
    }

    #[test]
    fn test_calculate_min_priority_fee() {
        let calculator = FeeCalculator::new();
        let default_fee = 1_000_000_000;
        let base_fee = BigInt::from(20_000_000_000u64);

        assert_eq!(
            calculator.calculate_min_priority_fee(&[0.9, 0.95, 1.0], &base_fee, default_fee).unwrap(),
            1_000_000_000
        );
        assert_eq!(
            calculator.calculate_min_priority_fee(&[0.7, 0.8, 0.75], &base_fee, default_fee).unwrap(),
            500_000_000
        );
        assert_eq!(
            calculator.calculate_min_priority_fee(&[0.1, 0.2, 0.3], &base_fee, default_fee).unwrap(),
            100_000_000
        );
        assert_eq!(calculator.calculate_min_priority_fee(&[], &base_fee, default_fee).unwrap(), 1_000_000_000);
    }

    #[test]
    fn test_calculate_priority_fees() {
        let calculator = FeeCalculator::new();
        let fee_history = create_test_fee_history();
        let priorities = [FeePriority::Slow, FeePriority::Normal, FeePriority::Fast];

        let result = calculator.calculate_priority_fees(&fee_history, &priorities, BigInt::from(100_000_000)).unwrap();

        assert_eq!(result.len(), 3);

        assert_eq!(result[0].priority, FeePriority::Slow);
        assert_eq!(result[0].value, BigInt::from(148893464));

        assert_eq!(result[1].priority, FeePriority::Normal);
        assert_eq!(result[1].value, BigInt::from(584926205));

        assert_eq!(result[2].priority, FeePriority::Fast);
        assert_eq!(result[2].value, BigInt::from(962019260));
    }

    #[test]
    fn test_calculate_priority_fees_errors() {
        let calculator = FeeCalculator::new();
        let empty_history = EthereumFeeHistory {
            reward: vec![],
            base_fee_per_gas: vec![],
            gas_used_ratio: vec![],
            oldest_block: 0,
        };

        assert!(calculator.calculate_priority_fees(&empty_history, &[FeePriority::Slow], BigInt::from(100)).is_err());
        assert!(calculator
            .calculate_priority_fees(&create_test_fee_history(), &[FeePriority::Slow, FeePriority::Normal], BigInt::from(100))
            .is_err());
    }
}
