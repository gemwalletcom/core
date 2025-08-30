use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{BroadcastOptions, Transaction, TransactionStateRequest, TransactionUpdate};

use crate::provider::transactions_mapper::{map_transaction_broadcast, map_transaction_status, map_transactions_by_address, map_transactions_by_block};
use crate::rpc::client::XRPClient;

#[async_trait]
impl<C: Client> ChainTransactions for XRPClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.broadcast_transaction(&data).await?;
        map_transaction_broadcast(&result)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let status = self.get_transaction_status(&request.id).await?;
        Ok(map_transaction_status(&status))
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let ledger = self.get_block_transactions(block as i64).await?;
        Ok(map_transactions_by_block(ledger))
    }

    async fn get_transactions_by_address(&self, address: String, limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let limit = limit.unwrap_or(100);
        let account_ledger = self.get_account_transactions(address, limit).await?;
        Ok(map_transactions_by_address(account_ledger))
    }
}
