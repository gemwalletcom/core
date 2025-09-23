use crate::models::Block;
use crate::models::fee::GasPrice;
use primitives::{FeePriority, FeeRate, GasPriceType, NodeSyncStatus};
use std::error::Error;

pub fn map_gas_price_to_priorities(gas_price: &GasPrice) -> Result<Vec<FeeRate>, Box<dyn std::error::Error + Sync + Send>> {
    let base_price = gas_price.gas_price;

    Ok(vec![
        FeeRate::new(FeePriority::Slow, GasPriceType::regular(base_price)),
        FeeRate::new(FeePriority::Normal, GasPriceType::regular(base_price)),
        FeeRate::new(FeePriority::Fast, GasPriceType::regular(base_price * 2)),
    ])
}

pub fn map_node_status(block: &Block) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    let current_block = Some(block.header.height);
    let latest_block_number = Some(block.header.height);
    Ok(NodeSyncStatus::new(true, latest_block_number, current_block))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::fee::GasPrice;
    use num_bigint::BigInt;
    use primitives::GasPriceType;

    #[test]
    fn test_map_gas_price_to_priorities() {
        let gas_price = GasPrice { gas_price: 1000000000 };

        let result = map_gas_price_to_priorities(&gas_price).unwrap();
        assert_eq!(result.len(), 3);
        match &result[0].gas_price_type {
            GasPriceType::Regular { gas_price } => assert_eq!(gas_price, &BigInt::from(1000000000u64)),
            _ => panic!("Expected Regular gas price"),
        }
        match &result[1].gas_price_type {
            GasPriceType::Regular { gas_price } => assert_eq!(gas_price, &BigInt::from(1000000000u64)),
            _ => panic!("Expected Regular gas price"),
        }
        match &result[2].gas_price_type {
            GasPriceType::Regular { gas_price } => assert_eq!(gas_price, &BigInt::from(2000000000u64)),
            _ => panic!("Expected Regular gas price"),
        }
    }

    #[test]
    fn test_map_node_status() {
        use crate::models::{Block, BlockHeader};

        let block = Block {
            header: BlockHeader {
                hash: String::new(),
                height: 123456789,
            },
        };
        let mapped = map_node_status(&block).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, Some(123456789));
        assert_eq!(mapped.current_block_number, Some(123456789));
    }
}
