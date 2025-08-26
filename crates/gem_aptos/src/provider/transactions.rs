use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use super::transactions_mapper::{map_transaction_broadcast, map_transaction_status};
use crate::{provider::transactions_mapper::map_transactions, rpc::client::AptosClient};

#[async_trait]
impl<C: Client> ChainTransactions for AptosClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.submit_transaction(&data).await?;
        map_transaction_broadcast(&result)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        Ok(map_transaction_status(&self.get_transaction_by_hash(&request.id).await?))
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(map_transactions(self.get_block_transactions(block).await?.transactions))
    }

    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(map_transactions(self.get_transactions_by_address(_address).await?))
    }
}
