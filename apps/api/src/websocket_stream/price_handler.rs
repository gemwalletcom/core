use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use cacher::CacheKey;
use pricer::PriceClient;
use primitives::{AssetId, AssetPrice, AssetPriceInfo, StreamEvent, StreamMessage, StreamMessagePrices, WebSocketPricePayload};
use redis::aio::MultiplexedConnection;
use rocket::tokio::sync::Mutex;

pub struct PriceHandler {
    price_client: Arc<Mutex<PriceClient>>,
    assets: HashSet<AssetId>,
    prices_to_publish: HashMap<String, AssetPrice>,
    interval: rocket::tokio::time::Interval,
}

impl PriceHandler {
    pub fn new(price_client: Arc<Mutex<PriceClient>>) -> Self {
        Self {
            price_client,
            assets: HashSet::new(),
            prices_to_publish: HashMap::new(),
            interval: rocket::tokio::time::interval(std::time::Duration::from_secs(5)),
        }
    }

    pub async fn next_interval(&mut self) {
        self.interval.tick().await;
    }

    pub fn take_prices(&mut self) -> Vec<AssetPrice> {
        self.prices_to_publish.drain().map(|(_, v)| v).collect()
    }

    fn get_channel_ids(&self) -> Vec<String> {
        self.assets.iter().map(|id| CacheKey::Price(&id.to_string()).key()).collect()
    }

    pub fn handle_price_message(&mut self, value: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let info = serde_json::from_slice::<AssetPriceInfo>(value)?;
        if !self.assets.contains(&info.asset_id) {
            return Ok(());
        }
        let price = info.as_asset_price_primitive();
        self.prices_to_publish.insert(price.asset_id.to_string(), price);
        Ok(())
    }

    pub async fn handle_stream_message(
        &mut self,
        message: &StreamMessage,
        redis_connection: &mut MultiplexedConnection,
    ) -> Result<Option<StreamEvent>, Box<dyn Error + Send + Sync>> {
        match message {
            StreamMessage::GetPrices(msg) => Ok(Some(self.get_prices(msg).await?)),
            StreamMessage::SubscribePrices(msg) => Ok(Some(self.subscribe_prices(msg, redis_connection).await?)),
            StreamMessage::AddPrices(msg) => Ok(Some(self.add_prices(msg, redis_connection).await?)),
            StreamMessage::UnsubscribePrices(msg) => Ok(Some(self.unsubscribe_prices(msg, redis_connection).await?)),
            StreamMessage::SubscribeRealtimePrices(_) | StreamMessage::UnsubscribeRealtimePrices(_) => Ok(None),
        }
    }

    async fn get_prices(&self, message: &StreamMessagePrices) -> Result<StreamEvent, Box<dyn Error + Send + Sync>> {
        self.price_event(message.assets.clone(), false).await
    }

    async fn subscribe_prices(&mut self, message: &StreamMessagePrices, redis_connection: &mut MultiplexedConnection) -> Result<StreamEvent, Box<dyn Error + Send + Sync>> {
        let old_channels = self.get_channel_ids();
        self.assets = message.assets.iter().cloned().collect();
        self.prices_to_publish.clear();
        if !old_channels.is_empty() {
            redis_connection.unsubscribe(old_channels).await?;
        }
        self.observe_assets().await;
        let event = self.price_event(self.assets.iter().cloned().collect(), true).await?;
        redis_connection.subscribe(self.get_channel_ids()).await?;
        Ok(event)
    }

    async fn add_prices(&mut self, message: &StreamMessagePrices, redis_connection: &mut MultiplexedConnection) -> Result<StreamEvent, Box<dyn Error + Send + Sync>> {
        let new_assets: Vec<AssetId> = message.assets.iter().filter(|asset| !self.assets.contains(*asset)).cloned().collect();
        let new_channels: Vec<String> = new_assets.iter().map(|id| CacheKey::Price(&id.to_string()).key()).collect();
        self.assets.extend(new_assets);
        self.observe_assets().await;
        let event = self.price_event(self.assets.iter().cloned().collect(), false).await?;
        if !new_channels.is_empty() {
            redis_connection.subscribe(new_channels).await?;
        }
        Ok(event)
    }

    async fn unsubscribe_prices(&mut self, message: &StreamMessagePrices, redis_connection: &mut MultiplexedConnection) -> Result<StreamEvent, Box<dyn Error + Send + Sync>> {
        let removed_assets: Vec<AssetId> = message.assets.iter().filter(|asset| self.assets.contains(*asset)).cloned().collect();
        let removed_channels: Vec<String> = removed_assets.iter().map(|id| CacheKey::Price(&id.to_string()).key()).collect();
        for asset in &removed_assets {
            self.assets.remove(asset);
            self.prices_to_publish.remove(&asset.to_string());
        }
        let event = self.price_event(self.assets.iter().cloned().collect(), false).await?;
        redis_connection.unsubscribe(removed_channels).await?;
        Ok(event)
    }

    async fn observe_assets(&self) {
        let observed: Vec<AssetId> = self.assets.iter().cloned().collect();
        let _ = self.price_client.lock().await.track_observed_assets(&observed).await;
    }

    async fn price_event(&self, asset_ids: Vec<AssetId>, include_rates: bool) -> Result<StreamEvent, Box<dyn Error + Send + Sync>> {
        let client = self.price_client.lock().await;
        let prices = client.get_cache_prices(asset_ids).await?.into_iter().map(|x| x.as_asset_price_primitive()).collect();
        let rates = if include_rates { client.get_cache_fiat_rates().await? } else { vec![] };
        Ok(StreamEvent::Prices(WebSocketPricePayload { prices, rates }))
    }
}
