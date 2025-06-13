use anyhow::{anyhow, Result};

use gem_evm::{ether_conv::EtherConv, jsonrpc::EthereumRpc};
use gemstone::ethereum::{
    calculator::GemFeeCalculator,
    model::{GemEthereumFeeHistory, GemFeePriority, GemPriorityFeeRecord},
};
use gemstone::network::{alien_provider::NativeProvider, JsonRpcClient};
use num_bigint::BigInt;
use num_traits::Num;
use primitives::Chain;
use std::fmt::Display;
use std::sync::Arc;

/// Represents unified gas fee data collected from a source.
#[derive(Debug)]
pub struct GemstoneFeeData {
    /// The latest block number.
    pub latest_block: u64,
    /// The suggested base fee in gwei.
    pub suggest_base_fee: String,
    /// Gas used ratio for the block, if available (e.g., "50.5%").
    pub gas_used_ratio: Option<String>,
    /// A list of priority fees for different priority levels.
    pub priority_fees: Vec<GemPriorityFeeRecord>,
}

impl Display for GemstoneFeeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block: {}, Base Fee: {}", self.latest_block, self.suggest_base_fee)?;
        for priority_fee in &self.priority_fees {
            write!(f, "{:?}: {}", priority_fee.priority, priority_fee.value)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct GemstoneClient {
    native_provider: Arc<NativeProvider>,
}

impl GemstoneClient {
    pub fn new(native_provider: Arc<NativeProvider>) -> Self {
        Self { native_provider }
    }

    pub async fn fetch_and_calculate_gemstone_fees(&self, blocks: u64, reward_percentiles: Vec<u64>) -> Result<GemstoneFeeData> {
        let client = JsonRpcClient::new_with_chain(self.native_provider.clone(), Chain::Ethereum);
        let call = EthereumRpc::FeeHistory { blocks, reward_percentiles };

        let resp = client.call(&call).await?;
        let fee_history_data: GemEthereumFeeHistory = resp.take()?;

        let oldest_block = u64::from_str_radix(&fee_history_data.oldest_block.replace("0x", ""), 16).expect("Invalid block number");

        let base_fee_for_next = fee_history_data
            .base_fee_per_gas
            .last()
            .ok_or_else(|| anyhow!("Fee history missing base_fee_per_gas data"))?;
        let base_fee_for_next = BigInt::from_str_radix(&base_fee_for_next.replace("0x", ""), 16).expect("Invalid base fee");

        let service = GemFeeCalculator::new();
        let min_priority_fee = 100000000; // 0.1 Gwei
        let priorities = vec![GemFeePriority::Slow, GemFeePriority::Normal, GemFeePriority::Fast];
        let mut calculated_priority_fees = service.calculate_priority_fees(fee_history_data.clone(), &priorities, min_priority_fee)?;
        calculated_priority_fees
            .iter_mut()
            .for_each(|fee| fee.value = EtherConv::to_gwei(&BigInt::from_str_radix(&fee.value, 10).expect("Invalid priority fee")));

        let gas_used_ratio_str = fee_history_data.gas_used_ratio.last().map(|val_ref| format!("{:.1}%", *val_ref * 100.0));

        Ok(GemstoneFeeData {
            latest_block: oldest_block + blocks - 1,
            suggest_base_fee: EtherConv::to_gwei(&base_fee_for_next),
            gas_used_ratio: gas_used_ratio_str,
            priority_fees: calculated_priority_fees,
        })
    }
}
