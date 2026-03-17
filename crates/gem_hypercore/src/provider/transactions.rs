use async_trait::async_trait;
use chain_traits::ChainTransactions;
use primitives::BroadcastOptions;
use std::error::Error;

use gem_client::Client;

use crate::{
    provider::transactions_mapper::{map_perpetual_fills, map_transaction_broadcast},
    rpc::client::HyperCoreClient,
};

#[async_trait]
impl<C: Client> ChainTransactions for HyperCoreClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let response = self.exchange(serde_json::from_str(&data)?).await?;
        map_transaction_broadcast(response, data)
    }

    async fn get_transactions_by_address(
        &self,
        address: String,
        _limit: Option<usize>,
        from_timestamp: Option<u64>,
    ) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Sync + Send>> {
        let start_time = from_timestamp.map(|ts| ts as i64 * 1000).unwrap_or(0);
        Ok(map_perpetual_fills(&address, self.get_user_fills_by_time(&address, start_time).await?))
    }
}
