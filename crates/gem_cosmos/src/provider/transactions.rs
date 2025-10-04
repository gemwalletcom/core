use async_trait::async_trait;
use chain_traits::ChainTransactions;
use futures::{StreamExt, TryStreamExt, stream};
use std::error::Error;

use gem_client::Client;
use primitives::{BroadcastOptions, Transaction};

use super::transactions_mapper::{map_transaction_broadcast, map_transaction_decode, map_transactions};
use crate::rpc::client::CosmosClient;

#[async_trait]
impl<C: Client> ChainTransactions for CosmosClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(map_transaction_broadcast(&self.broadcast_transaction(&data).await?)?)
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let response = self.get_block(block.to_string().as_str()).await?;
        let transaction_ids = response
            .block
            .data
            .txs
            .clone()
            .into_iter()
            .filter(|x| x.len() < 1024) // parse only transfer / stake transactions, ideally filter by type
            .flat_map(map_transaction_decode)
            .collect::<Vec<_>>();

        let receipts = stream::iter(transaction_ids)
            .map(|txid| async move { self.get_transaction(txid.clone()).await })
            .buffer_unordered(5)
            .try_collect()
            .await?;

        Ok(map_transactions(self.chain, receipts))
    }

    async fn get_transactions_by_address(&self, address: String, limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let limit = limit.unwrap_or(20);
        let transactions = self.get_transactions_by_address_with_limit(&address, limit).await?;
        Ok(map_transactions(self.chain, transactions))
    }
}
