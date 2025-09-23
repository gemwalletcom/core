use std::error::Error;

use gem_evm::fee_calculator::FeeCalculator;
use gem_evm::models::fee::EthereumFeeHistory;
use gem_evm::{ether_conv::EtherConv, jsonrpc::EthereumRpc};
use gemstone::network::{alien_provider::NativeProvider, jsonrpc_client_with_chain};
use num_bigint::BigInt;
use primitives::{Chain, PriorityFeeValue, fee::FeePriority};
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
    pub priority_fees: Vec<PriorityFeeValue>,
}

impl Display for GemstoneFeeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block: {}, Base Fee: {}", self.latest_block, self.suggest_base_fee)?;
        for priority_fee in &self.priority_fees {
            write!(f, "{:?}: {}", priority_fee.priority, EtherConv::to_gwei(&priority_fee.value))?;
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

    pub async fn fetch_base_priority_fees(
        &self,
        blocks: u64,
        reward_percentiles: Vec<u64>,
        min_priority_fee: u64,
    ) -> Result<GemstoneFeeData, Box<dyn Error + Send + Sync>> {
        let client = jsonrpc_client_with_chain(self.native_provider.clone(), Chain::Ethereum);
        let call = EthereumRpc::FeeHistory { blocks, reward_percentiles };

        let fee_history_data: EthereumFeeHistory = client.request(call).await?;

        let base_fee_for_next = fee_history_data.base_fee_per_gas.last().ok_or("Fee history missing base_fee_per_gas data")?;

        let service = FeeCalculator::new();
        let priorities = vec![FeePriority::Slow, FeePriority::Normal, FeePriority::Fast];
        let calculated_priority_fees = service
            .calculate_priority_fees(&fee_history_data, &priorities, BigInt::from(min_priority_fee))
            .map_err(|e| format!("Failed to calculate priority fees: {}", e))?;

        let gas_used_ratio = fee_history_data.gas_used_ratio.last().map(|val_ref| format!("{:.1}%", *val_ref * 100.0));

        Ok(GemstoneFeeData {
            latest_block: fee_history_data.oldest_block + blocks - 1,
            suggest_base_fee: EtherConv::to_gwei(base_fee_for_next),
            gas_used_ratio,
            priority_fees: calculated_priority_fees,
        })
    }
}
