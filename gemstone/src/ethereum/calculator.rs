use alloy_primitives::U256;
use primitives::{Chain, EVMChain};

use super::model::{GemEthereumFeeHistory, GemFeePriority, GemPriorityFeeRecord};
use crate::{config::evm_chain::get_evm_chain_config, GemstoneError};
use gem_evm::parse_u256;

#[derive(Debug, Default, uniffi::Object)]
pub struct GemFeeCalculator {}

impl GemFeeCalculator {
    pub fn calculate_min_priority_fee(&self, gas_used_ratios: Vec<f64>, base_fee: &str, default_min_priority_fee: u64) -> Result<u64, GemstoneError> {
        let base_fee = parse_u256(base_fee).ok_or_else(|| GemstoneError::AnyError {
            msg: format!("Invalid base_fee: {base_fee}"),
        })?;

        if gas_used_ratios.is_empty() || base_fee == U256::ZERO {
            return Ok(default_min_priority_fee);
        }
        let avg_ratio: f64 = gas_used_ratios.iter().sum::<f64>() / gas_used_ratios.len() as f64;

        let result = match avg_ratio {
            r if r >= 0.9 => default_min_priority_fee,     // congested
            r if r >= 0.7 => default_min_priority_fee / 2, // moderate
            _ => default_min_priority_fee / 10,            // quiet
        };
        Ok(result)
    }

    pub fn calculate_priority_fees(
        &self,
        fee_history: GemEthereumFeeHistory,
        priorities: &[GemFeePriority],
        min_priority_fee: u64,
    ) -> Result<Vec<GemPriorityFeeRecord>, GemstoneError> {
        if fee_history.reward.is_empty() {
            return Err(GemstoneError::AnyError {
                msg: "fee_history.reward is empty".into(),
            });
        }

        if priorities.len() != fee_history.reward[0].len() {
            return Err(GemstoneError::AnyError {
                msg: "priorities.len() != fee_history.reward[0].len()".into(),
            });
        }

        let rewards = &fee_history.reward;

        let mut columns: Vec<Vec<U256>> = vec![Vec::new(); priorities.len()];
        for row in rewards {
            for (i, hex_fee) in row.iter().enumerate().take(priorities.len()) {
                if let Some(bn) = parse_u256(hex_fee) {
                    columns[i].push(bn);
                }
            }
        }

        let result = priorities
            .iter()
            .zip(columns.iter())
            .map(|(&priority, fees)| {
                let value = if fees.is_empty() {
                    // no data → use min
                    U256::from(min_priority_fee).to_string()
                } else {
                    // sum, average = sum / count, then max(min, avg)
                    let sum = fees.iter().cloned().fold(U256::ZERO, |a, b| a + b);
                    let avg = sum / U256::from(fees.len());
                    let min_value = U256::from(min_priority_fee);
                    let chosen = if avg < min_value { min_value } else { avg };
                    chosen.to_string()
                };

                GemPriorityFeeRecord { priority, value }
            })
            .collect();

        Ok(result)
    }
}

#[uniffi::export]
impl GemFeeCalculator {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calculate_base_priority_fees(&self, chain: Chain, history: GemEthereumFeeHistory) -> Result<Vec<GemPriorityFeeRecord>, GemstoneError> {
        let evm_chain = EVMChain::from_chain(chain).ok_or(GemstoneError::AnyError { msg: "Invalid chain".into() })?;
        let config = get_evm_chain_config(evm_chain);
        let base_fee = history.base_fee_per_gas.last().ok_or(GemstoneError::AnyError {
            msg: "Invalid base fee".into(),
        })?;

        let min_priority_fee = self.calculate_min_priority_fee(history.gas_used_ratio.clone(), base_fee, config.min_priority_fee)?;
        let priorities = vec![GemFeePriority::Slow, GemFeePriority::Normal, GemFeePriority::Fast];
        self.calculate_priority_fees(history, &priorities, min_priority_fee)
    }
}

#[cfg(test)]
mod tests {
    use crate::network::JsonRpcResponse;

    use super::*;

    #[test]
    fn test_calculate_min_priority_fee_logic() {
        let service = GemFeeCalculator::new();
        let default_fee = 1_000_000_000;
        let base_fee = "0x4a817c800".to_string(); // 20 Gwei

        // Congested
        let ratios_congested = vec![0.9, 0.95, 1.0];
        assert_eq!(
            service.calculate_min_priority_fee(ratios_congested, &base_fee, default_fee).unwrap(),
            1000000000
        );

        // Moderate
        let ratios_moderate = vec![0.7, 0.8, 0.75];
        assert_eq!(service.calculate_min_priority_fee(ratios_moderate, &base_fee, default_fee).unwrap(), 500000000);

        // Quiet
        let ratios_quiet = vec![0.1, 0.2, 0.3];
        assert_eq!(service.calculate_min_priority_fee(ratios_quiet, &base_fee, default_fee).unwrap(), 100000000);

        // Empty ratios
        assert_eq!(service.calculate_min_priority_fee(vec![], &base_fee, default_fee).unwrap(), 1000000000);
    }

    #[test]
    fn test_calculate_priority_fees_logic() {
        let service = GemFeeCalculator::new();
        let min_priority_fee = 100000000; // 0.1 Gwei

        let json_str = include_str!("./test/fee_history.json");
        let response: JsonRpcResponse<GemEthereumFeeHistory> = serde_json::from_str(json_str).unwrap();
        let fee_history: GemEthereumFeeHistory = response.result;

        let priorities = vec![GemFeePriority::Slow, GemFeePriority::Normal, GemFeePriority::Fast];
        let result = service.calculate_priority_fees(fee_history, &priorities, min_priority_fee).unwrap();

        assert_eq!(result.len(), 3);

        let slow_fee = &result[0];
        let normal_fee = &result[1];
        let fast_fee = &result[2];

        assert_eq!(slow_fee.value, "148893464");
        assert_eq!(normal_fee.value, "584926205");
        assert_eq!(fast_fee.value, "962019260");
    }
}
