use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use primitives::{TransactionStateRequest, TransactionUpdate};
use std::error::Error;

use gem_client::Client;

use crate::{provider::transaction_state_mapper::map_transaction_status, rpc::client::PolkadotClient};

#[async_trait]
impl<C: Client> ChainTransactionState for PolkadotClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let block_number = request.block_number;
        if block_number <= 0 {
            return Err("Invalid block number".into());
        }

        let block_head = self.get_block_head().await?;
        let from_block = block_number as u64;
        let to_block = calculate_to_block(block_head.number, from_block);

        let blocks = self.get_blocks(&from_block.to_string(), &to_block.to_string()).await?;
        Ok(map_transaction_status(blocks, &request.id, block_number))
    }
}

fn calculate_to_block(block_head_number: u64, from_block: u64) -> u64 {
    let to_block = std::cmp::min(block_head_number, from_block + 64);
    std::cmp::max(to_block, from_block + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_to_block() {
        assert_eq!(calculate_to_block(100, 100), 101);
        assert_eq!(calculate_to_block(200, 100), 164);
        assert_eq!(calculate_to_block(105, 100), 105);
        assert_eq!(calculate_to_block(50, 100), 101);
    }
}
