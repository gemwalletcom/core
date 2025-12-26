use std::error::Error;

use primitives::{AssetId, Chain};

use crate::{
    AssetsAddressPayload, ChainAddressPayload, ChartsPayload, ExchangeName, FetchAssetsPayload, FetchBlocksPayload, NotificationsFailedPayload,
    NotificationsPayload, PricesPayload, QueueName, RewardsNotificationPayload, RewardsRedemptionPayload, StreamProducer, TransactionsPayload,
};

#[async_trait::async_trait]
pub trait StreamProducerQueue {
    async fn publish_fetch_assets(&self, asset_ids: Vec<AssetId>) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_transactions(&self, payload: TransactionsPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_notifications_transactions(&self, payload: Vec<NotificationsPayload>) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_notifications_price_alerts(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_notifications_observers(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_notifications_support(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_notifications_rewards(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_rewards_events(&self, payload: Vec<RewardsNotificationPayload>) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_rewards_redemption(&self, payload: RewardsRedemptionPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_notifications_failed(&self, payload: NotificationsFailedPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_store_assets_addresses_associations(&self, payload: AssetsAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_prices(&self, payload: PricesPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_charts(&self, payload: ChartsPayload) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn publish_blocks(&self, chain: Chain, blocks: &[u64]) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn publish_new_addresses(&self, payload: Vec<ChainAddressPayload>) -> Result<bool, Box<dyn Error + Send + Sync>>;
}

#[async_trait::async_trait]
impl StreamProducerQueue for StreamProducer {
    async fn publish_fetch_assets(&self, asset_ids: Vec<AssetId>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        for asset_id in &asset_ids {
            let payload = FetchAssetsPayload::new(asset_id.clone());
            self.publish(QueueName::FetchAssets, &payload).await?;
        }
        Ok(true)
    }

    async fn publish_transactions(&self, payload: TransactionsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.transactions.is_empty() {
            return Ok(true);
        }
        self.publish(QueueName::StoreTransactions, &payload).await
    }

    async fn publish_notifications_transactions(&self, payload: Vec<NotificationsPayload>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let payload: Vec<NotificationsPayload> = payload.into_iter().filter(|p| !p.notifications.is_empty()).collect();
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

    async fn publish_notifications_rewards(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.notifications.is_empty() {
            return Ok(true);
        }
        self.publish(QueueName::NotificationsRewards, &payload).await
    }

    async fn publish_rewards_events(&self, payload: Vec<RewardsNotificationPayload>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.is_empty() {
            return Ok(true);
        }
        self.publish_batch(QueueName::RewardsEvents, &payload).await
    }

    async fn publish_rewards_redemption(&self, payload: RewardsRedemptionPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.publish(QueueName::RewardsRedemptions, &payload).await
    }

    async fn publish_notifications_failed(&self, payload: NotificationsFailedPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.failures.is_empty() {
            return Ok(true);
        }
        self.publish(QueueName::NotificationsFailed, &payload).await
    }

    async fn publish_store_assets_addresses_associations(&self, payload: AssetsAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.values.is_empty() {
            return Ok(true);
        }
        self.publish(QueueName::StoreAssetsAssociations, &payload).await
    }

    async fn publish_prices(&self, payload: PricesPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.prices.is_empty() {
            return Ok(true);
        }
        self.publish(QueueName::StorePrices, &payload).await
    }

    async fn publish_charts(&self, payload: ChartsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if payload.charts.is_empty() {
            return Ok(true);
        }
        self.publish(QueueName::StoreCharts, &payload).await
    }

    async fn publish_blocks(&self, chain: Chain, blocks: &[u64]) -> Result<(), Box<dyn Error + Send + Sync>> {
        for block in blocks {
            let payload = FetchBlocksPayload::new(chain, *block);
            self.publish_with_routing_key(QueueName::FetchBlocks, chain.as_ref(), &payload).await?;
        }
        Ok(())
    }

    async fn publish_new_addresses(&self, payload: Vec<ChainAddressPayload>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        for item in &payload {
            self.publish_to_exchange_with_routing_key(ExchangeName::NewAddresses, item.value.chain.as_ref(), item)
                .await?;
        }
        Ok(true)
    }
}
