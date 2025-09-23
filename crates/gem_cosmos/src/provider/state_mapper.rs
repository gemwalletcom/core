use num_bigint::BigInt;
use primitives::{FeePriority, FeeRate, GasPriceType, NodeSyncStatus, chain_cosmos::CosmosChain};
use std::error::Error;

pub fn calculate_fee_rates(chain: CosmosChain, base_fee: BigInt) -> Vec<FeeRate> {
    match chain {
        CosmosChain::Thorchain => {
            vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(base_fee))]
        }
        CosmosChain::Cosmos | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Sei | CosmosChain::Injective | CosmosChain::Noble => {
            vec![
                FeeRate::new(FeePriority::Normal, GasPriceType::regular(base_fee.clone())),
                FeeRate::new(FeePriority::Fast, GasPriceType::regular(&base_fee * BigInt::from(2))),
            ]
        }
    }
}

pub fn map_node_status(latest_block: u64) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    let current_block = Some(latest_block);
    let latest_block_number = Some(latest_block);
    Ok(NodeSyncStatus::new(true, latest_block_number, current_block))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_node_status() {
        let latest_block = 12345678u64;
        let mapped = map_node_status(latest_block).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, Some(12345678));
        assert_eq!(mapped.current_block_number, Some(12345678));
    }
}
