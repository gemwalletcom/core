use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest};
use std::error::Error;

use gem_client::Client;

use crate::{provider::transactions_mapper::map_user_fills, rpc::client::HyperCoreClient};

#[async_trait]
impl<C: Client> ChainTransactions for HyperCoreClient<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Sync + Send>> {
        let start_time = request.from_timestamp.map(|ts| ts as i64 * 1000).unwrap_or(0);
        let fills = self.get_user_fills_by_time(&request.address, start_time).await?;
        let spot_meta = if fills.iter().any(|fill| fill.coin.starts_with('@')) {
            Some(self.get_spot_meta().await?)
        } else {
            None
        };
        let transactions = map_user_fills(&request.address, fills, spot_meta.as_ref());

        match request.asset_id {
            Some(asset_id) => Ok(transactions.into_iter().filter(|tx| tx.asset_ids().contains(&asset_id)).collect()),
            None => Ok(transactions),
        }
    }
}
