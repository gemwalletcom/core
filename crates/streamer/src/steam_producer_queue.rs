use std::error::Error;

use primitives::AssetId;

use crate::{AssetsAddressPayload, FetchAssetsPayload, NotificationsPayload, QueueName, StreamProducer, TransactionsPayload};

#[async_trait::async_trait]
pub trait StreamProducerQueue {
    async fn publish_fetch_assets(&self, asset_ids: Vec<AssetId>) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_transactions(&self, payload: TransactionsPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_notifications_transactions(&self, payload: Vec<NotificationsPayload>) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_store_assets_addresses_associations(&self, payload: Vec<AssetsAddressPayload>) -> Result<bool, Box<dyn Error + Send + Sync>>;
}

#[async_trait::async_trait]
impl StreamProducerQueue for StreamProducer {
    async fn publish_fetch_assets(&self, asset_ids: Vec<AssetId>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.publish_batch(
            QueueName::FetchAssets,
            &asset_ids.iter().map(|x| FetchAssetsPayload::new(x.clone())).collect::<Vec<_>>(),
        )
        .await
    }

    async fn publish_transactions(&self, payload: TransactionsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.publish(QueueName::Transactions, &payload).await
    }

    async fn publish_notifications_transactions(&self, payload: Vec<NotificationsPayload>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.publish_batch(QueueName::NotificationsTransactions, &payload).await
    }

    async fn publish_store_assets_addresses_associations(&self, payload: Vec<AssetsAddressPayload>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.publish_batch(QueueName::StoreAssetsAddressesAssociations, &payload).await
    }
}
