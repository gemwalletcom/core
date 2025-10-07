use std::error::Error;

use primitives::AssetId;

use crate::{AssetsAddressPayload, FetchAssetsPayload, NotificationsPayload, QueueName, StreamProducer, TransactionsPayload};

#[async_trait::async_trait]
pub trait StreamProducerQueue {
    async fn publish_fetch_assets(&self, asset_ids: Vec<AssetId>) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_transactions(&self, payload: TransactionsPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_notifications_transactions(&self, payload: Vec<NotificationsPayload>) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_notifications_price_alerts(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_notifications_observers(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_notifications_support(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_store_assets_addresses_associations(&self, payload: Vec<AssetsAddressPayload>) -> Result<bool, Box<dyn Error + Send + Sync>>;
}

#[async_trait::async_trait]
impl StreamProducerQueue for StreamProducer {
    async fn publish_fetch_assets(&self, asset_ids: Vec<AssetId>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let payload = asset_ids.iter().map(|x| FetchAssetsPayload::new(x.clone())).collect::<Vec<_>>();
        if payload.is_empty() {
            return Ok(true);
        }
        self.publish_batch(QueueName::FetchAssets, &payload).await
    }

    async fn publish_transactions(&self, payload: TransactionsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.transactions.is_empty() {
            return Ok(true);
        }
        self.publish(QueueName::StoreTransactions, &payload).await
    }

    async fn publish_notifications_transactions(&self, payload: Vec<NotificationsPayload>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.is_empty() {
            return Ok(true);
        }
        self.publish_batch(QueueName::NotificationsTransactions, &payload).await
    }

    async fn publish_notifications_price_alerts(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.notifications.is_empty() {
            return Ok(true);
        }
        self.publish(QueueName::NotificationsPriceAlerts, &payload).await
    }

    async fn publish_notifications_observers(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.notifications.is_empty() {
            return Ok(true);
        }
        self.publish(QueueName::NotificationsObservers, &payload).await
    }

    async fn publish_notifications_support(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.notifications.is_empty() {
            return Ok(true);
        }
        self.publish(QueueName::NotificationsSupport, &payload).await
    }

    async fn publish_store_assets_addresses_associations(&self, payload: Vec<AssetsAddressPayload>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.is_empty() {
            return Ok(true);
        }
        self.publish_batch(QueueName::StoreAssetsAssociations, &payload).await
    }
}
