use async_trait::async_trait;
use chain_traits::ChainTransactions;
use futures::future;
use std::error::Error;

use gem_client::Client;
use primitives::{BroadcastOptions, Transaction, TransactionStateRequest, TransactionUpdate};

use super::transactions_mapper::{map_transaction_broadcast, map_transaction_decode, map_transaction_status, map_transactions};
use crate::rpc::client::CosmosClient;

#[async_trait]
impl<C: Client> ChainTransactions for CosmosClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(map_transaction_broadcast(&self.broadcast_transaction(&data).await?)?)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        Ok(map_transaction_status(self.get_transaction(request.id).await?))
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let response = self.get_block(block.to_string().as_str()).await?;
        let transaction_ids = response.block.data.txs.clone().into_iter().flat_map(map_transaction_decode).collect::<Vec<_>>();
        let receipts = future::try_join_all(transaction_ids.into_iter().map(|x| self.get_transaction(x))).await?;

        Ok(map_transactions(self.get_chain().as_chain(), receipts))
    }

    async fn get_transactions_by_address(&self, address: String, limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let limit = limit.unwrap_or(20);
        let transactions = self.get_transactions_by_address_with_limit(&address, limit).await?;
        Ok(map_transactions(self.get_chain().as_chain(), transactions))
    }
}
