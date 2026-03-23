use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use crate::provider::transactions_mapper::{map_transactions_by_address, map_transactions_by_block};
use crate::rpc::client::XRPClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactions for XRPClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let ledger = self.get_block_transactions(block).await?;
        Ok(map_transactions_by_block(ledger))
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let limit = limit.unwrap_or(100);
        let account_ledger = self.get_account_transactions(address, limit).await?;
        Ok(map_transactions_by_address(account_ledger))
    }
}
