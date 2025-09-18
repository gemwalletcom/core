use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{BroadcastOptions, Transaction, TransactionStateRequest, TransactionUpdate};

use crate::{provider::transactions_mapper::map_transaction_status, rpc::client::NearClient};

#[async_trait]
impl<C: Client + Clone> ChainTransactions for NearClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.broadcast_transaction(&data).await?.transaction.hash)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let result = self.get_transaction_status(&request.id, &request.sender_address).await?;
        Ok(map_transaction_status(&result))
    }

    async fn get_transactions_by_block(&self, _block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        // TODO: Implement NEAR block transaction fetching
        // NEAR requires calling EXPERIMENTAL_chunk or similar RPC methods to get block transactions
        // This would need additional RPC methods and transaction mapping logic
        Ok(vec![])
    }

    async fn get_transactions_by_address(&self, _address: String, _limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}
