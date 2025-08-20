use async_trait::async_trait;
use chain_traits::ChainState;
use gem_client::Client;
use number_formatter::BigNumberFormatter;
use primitives::fee::{FeePriority, FeeRate};
use num_bigint::BigInt;
use std::error::Error;

use crate::rpc::client::BitcoinClient;

#[async_trait]
impl<C: Client> ChainState for BitcoinClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        let block = self.get_block_info(1).await?;
        block.previous_block_hash.ok_or_else(|| "Unable to get block hash".into())
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        let node_info = self.get_node_info().await?;
        Ok(node_info.blockbook.best_height)
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let priority = self.chain.get_blocks_fee_priority();
        let (slow, normal, fast) = futures::try_join!(self.get_fee(priority.slow), self.get_fee(priority.normal), self.get_fee(priority.fast))?;
        Ok(vec![
            FeeRate::regular(FeePriority::Slow, slow),
            FeeRate::regular(FeePriority::Normal, normal),
            FeeRate::regular(FeePriority::Fast, fast),
        ])
    }
}

impl<C: Client> BitcoinClient<C> {
    async fn get_fee(&self, blocks: i32) -> Result<BigInt, Box<dyn Error + Sync + Send>> {
        let fee_sat_per_kb = self.get_fee_priority(blocks).await?;
        Ok(calculate_fee_rate(&fee_sat_per_kb, self.chain.minimum_byte_fee() as u32)?)
    }
}

fn calculate_fee_rate(fee_sat_per_kb: &str, minimum_byte_fee: u32) -> Result<BigInt, Box<dyn Error + Sync + Send>> {
    let rate = BigNumberFormatter::value_from_amount(fee_sat_per_kb, 8)?.parse::<f64>()? / 1000.0;
    let minimum_byte_fee = minimum_byte_fee as f64;
    
    Ok(BigInt::from(rate.max(minimum_byte_fee) as i64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fee_rate() {
        // Test with specific Bitcoin fee values (BTC amounts converted to sat/vB)
        assert_eq!(calculate_fee_rate("0.00002132", 1).unwrap(), BigInt::from(2));
        assert_eq!(calculate_fee_rate("0.00001083", 1).unwrap(), BigInt::from(1));
        
        // Test minimum enforcement
        assert_eq!(calculate_fee_rate("0.00000500", 10).unwrap(), BigInt::from(10));
        
        // Test higher fee rates
        assert_eq!(calculate_fee_rate("0.00100000", 1).unwrap(), BigInt::from(100));
    }
}