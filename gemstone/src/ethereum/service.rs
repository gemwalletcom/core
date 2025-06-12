use super::model::{EvmFeeParams, GemBasePriorityFees, GemEthereumFeeHistory, GemPriorityFeeRecord};
use crate::network::AlienError;
use crate::{
    ethereum::model::EVMHistoryRewardPercentiles,
    network::{AlienProvider, JsonRpcClient, JsonRpcResult},
};
use alloy_primitives::U256;
use gem_evm::{jsonrpc::EthereumRpc, parse_u256};
use primitives::fee::FeePriority;
use std::{cmp::max, sync::Arc};

#[derive(Debug, uniffi::Object)]
pub struct GemEthereumService {
    provider: Arc<dyn AlienProvider>,
}

impl GemEthereumService {
    fn calculate_min_priority_fee(&self, gas_used_ratios: &[f64], base_fee: U256, default_min_priority_fee: U256) -> U256 {
        if gas_used_ratios.is_empty() || base_fee == U256::ZERO {
            return default_min_priority_fee;
        }
        let avg_ratio: f64 = gas_used_ratios.iter().sum::<f64>() / gas_used_ratios.len() as f64;

        match avg_ratio {
            r if r >= 0.9 => default_min_priority_fee,                 // congested
            r if r >= 0.7 => default_min_priority_fee / U256::from(2), // moderate
            _ => default_min_priority_fee / U256::from(10),            // quiet
        }
    }

    fn calculate_priority_fees(
        &self,
        rewards: &[Vec<U256>],
        rewards_percentiles: &EVMHistoryRewardPercentiles,
        min_priority_fee: U256,
    ) -> Result<Vec<GemPriorityFeeRecord>, AlienError> {
        let percentile_to_priority = [
            (rewards_percentiles.slow, FeePriority::Slow),
            (rewards_percentiles.normal, FeePriority::Normal),
            (rewards_percentiles.fast, FeePriority::Fast),
        ];

        let sorted_percentiles = rewards_percentiles.all();

        percentile_to_priority
            .into_iter()
            .map(|(percentile_val, priority)| {
                // This should always succeed as `sorted_percentiles` is built from the same source.
                let index = sorted_percentiles
                    .iter()
                    .position(|p| p == &percentile_val)
                    .ok_or_else(|| AlienError::ResponseError {
                        msg: format!("Percentile {} not found in sorted list", percentile_val),
                    })?;

                let fees_for_percentile: Vec<U256> = rewards.iter().filter_map(|block_rewards| block_rewards.get(index).cloned()).collect();

                let average = if fees_for_percentile.is_empty() {
                    min_priority_fee
                } else {
                    let sum: U256 = fees_for_percentile.iter().sum();
                    max(min_priority_fee, sum / U256::from(fees_for_percentile.len() as u64))
                };

                Ok(GemPriorityFeeRecord {
                    priority,
                    value: average.to_string(),
                })
            })
            .collect()
    }
}

#[uniffi::export]
impl GemEthereumService {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_base_priority_fees(&self, params: EvmFeeParams) -> Result<GemBasePriorityFees, AlienError> {
        let parse_hex = |hex: &str, context: &str| {
            parse_u256(hex).ok_or_else(|| AlienError::ResponseError {
                msg: format!("Invalid {} hex: {}", context, hex),
            })
        };

        let client = JsonRpcClient::new_with_chain(self.provider.clone(), params.chain);
        let call = EthereumRpc::FeeHistory {
            blocks: params.history_blocks,
            reward_percentiles: params.reward_percentiles.all(),
        };

        let resp: JsonRpcResult<GemEthereumFeeHistory> = client.call(&call).await?;
        let fee_history_data = resp.take()?;

        let base_fee_per_gas_hex = fee_history_data.base_fee_per_gas.last().ok_or_else(|| AlienError::ResponseError {
            msg: "Unable to retrieve base fee from history".to_string(),
        })?;

        let base_fee_per_gas = parse_hex(&base_fee_per_gas_hex.to_string(), "base_fee_per_gas")?;
        let default_min_priority_fee = parse_hex(&params.min_priority_fee, "min_priority_fee")?;

        let actual_min_priority_fee = self.calculate_min_priority_fee(&fee_history_data.gas_used_ratio, base_fee_per_gas, default_min_priority_fee);

        let rewards_u256: Result<Vec<Vec<U256>>, AlienError> = fee_history_data
            .reward
            .iter()
            .map(|block_rewards_str| {
                block_rewards_str
                    .iter()
                    .map(|r_str| parse_hex(r_str, "reward"))
                    .collect::<Result<Vec<U256>, _>>()
            })
            .collect();
        let rewards_u256 = rewards_u256?;

        let priority_fees = self.calculate_priority_fees(&rewards_u256, &params.reward_percentiles, actual_min_priority_fee)?;

        Ok(GemBasePriorityFees {
            base_fee: base_fee_per_gas.to_string(),
            priority_fees,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::mock::AlienProviderMock;
    use primitives::Chain;

    #[test]
    fn test_calculate_min_priority_fee_logic() {
        let service = GemEthereumService::new(Arc::new(AlienProviderMock::new("".to_string())));
        let default_fee = U256::from(1_000_000_000u64); // 1 Gwei
        let base_fee = U256::from(20_000_000_000u64); // 20 Gwei

        // Congested
        let ratios_congested = vec![0.9, 0.95, 1.0];
        assert_eq!(service.calculate_min_priority_fee(&ratios_congested, base_fee, default_fee), default_fee);

        // Moderate
        let ratios_moderate = vec![0.7, 0.8, 0.75];
        assert_eq!(
            service.calculate_min_priority_fee(&ratios_moderate, base_fee, default_fee),
            default_fee / U256::from(2)
        );

        // Quiet
        let ratios_quiet = vec![0.1, 0.2, 0.3];
        assert_eq!(
            service.calculate_min_priority_fee(&ratios_quiet, base_fee, default_fee),
            default_fee / U256::from(10)
        );

        // Empty ratios
        assert_eq!(service.calculate_min_priority_fee(&[], base_fee, default_fee), default_fee);
    }

    #[test]
    fn test_calculate_priority_fees_logic() {
        let service = GemEthereumService::new(Arc::new(AlienProviderMock::new("".to_string())));
        let rewards_percentiles = EVMHistoryRewardPercentiles {
            slow: 10,
            normal: 50,
            fast: 90,
        };
        let min_priority_fee = U256::from(1_000_000_000u64); // 1 Gwei

        let rewards = vec![
            vec![U256::from(1_100_000_000u64), U256::from(2_000_000_000u64), U256::from(3_000_000_000u64)],
            vec![U256::from(900_000_000u64), U256::from(2_200_000_000u64), U256::from(3_500_000_000u64)],
        ];

        let result = service.calculate_priority_fees(&rewards, &rewards_percentiles, min_priority_fee).unwrap();

        let slow_fee = result.iter().find(|f| f.priority == FeePriority::Slow).unwrap();
        let normal_fee = result.iter().find(|f| f.priority == FeePriority::Normal).unwrap();
        let fast_fee = result.iter().find(|f| f.priority == FeePriority::Fast).unwrap();

        assert_eq!(slow_fee.value, "1000000000");
        assert_eq!(normal_fee.value, "2100000000");
        assert_eq!(fast_fee.value, "3250000000");
    }

    #[tokio::test]
    async fn test_get_base_priority_fees_full() {
        let mock_response = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "baseFeePerGas": ["0x75bcd15", "0x7735940"],
                "gasUsedRatio": [0.1, 0.2],
                "reward": [
                    ["0x3b9aca00", "0x77359400", "0xb2d05e00"],
                    ["0x3b9aca00", "0x77359400", "0xb2d05e00"]
                ]
            }
        }"#
        .to_string();

        let mock_provider = Arc::new(AlienProviderMock::new(mock_response));
        let service = GemEthereumService::new(mock_provider);

        let params = EvmFeeParams {
            history_blocks: 2,
            reward_percentiles: EVMHistoryRewardPercentiles {
                slow: 10,
                normal: 50,
                fast: 90,
            },
            min_priority_fee: "0x3b9aca00".to_string(), // 1 Gwei
            chain: Chain::Ethereum,
        };

        let result = service.get_base_priority_fees(params).await.unwrap();

        assert_eq!(result.base_fee, "125000000"); // 0x7735940
        let slow_fee = result.priority_fees.iter().find(|f| f.priority == FeePriority::Slow).unwrap();
        let normal_fee = result.priority_fees.iter().find(|f| f.priority == FeePriority::Normal).unwrap();
        let fast_fee = result.priority_fees.iter().find(|f| f.priority == FeePriority::Fast).unwrap();

        assert_eq!(slow_fee.value, "1000000000");
        assert_eq!(normal_fee.value, "2000000000");
        assert_eq!(fast_fee.value, "3000000000");
    }
}
