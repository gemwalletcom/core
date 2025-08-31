use async_trait::async_trait;
use chain_traits::{ChainProvider, ChainTransactions};
use std::error::Error;

use gem_client::Client;
use primitives::{BroadcastOptions, Transaction, TransactionStateRequest, TransactionUpdate};

use super::transactions_mapper;
use crate::rpc::client::PolkadotClient;

#[async_trait]
impl<C: Client> ChainTransactions for PolkadotClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let response = self.broadcast_transaction(data).await?;

        if let Some(hash) = response.hash {
            Ok(hash)
        } else if let Some(error) = response.error {
            let cause = response.cause.unwrap_or_default();
            Err(format!("{}: {}", error, cause).into())
        } else {
            Err("Invalid broadcast response".into())
        }
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let block_number = request.block_number;
        if block_number <= 0 {
            return Err("Invalid block number".into());
        }

        let block_head = self.get_block_head().await?;
        let from_block = block_number as u64;
        let to_block = calculate_to_block(block_head.number, from_block);

        let blocks = self.get_blocks(&from_block.to_string(), &to_block.to_string()).await?;
        Ok(transactions_mapper::map_transaction_status(blocks, &request.id, block_number))
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block_data = self.get_block(block as i64).await?;
        Ok(transactions_mapper::map_transactions(self.get_chain(), block_data))
    }

    async fn get_transactions_by_address(&self, _address: String, _limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
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
