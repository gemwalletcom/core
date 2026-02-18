use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use pricer::PriceClient;
use primitives::{AssetId, AssetPrice, AssetPriceInfo, StreamEvent, StreamMessage, WebSocketPricePayload, asset::AssetHashSetExt};
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
        self.assets.iter().map(|id| id.to_string()).collect()
    }

    pub fn handle_price_message(&mut self, value: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let info = serde_json::from_slice::<AssetPriceInfo>(value)?;
        let price = info.as_asset_price_primitive();
        self.prices_to_publish.insert(price.asset_id.to_string(), price);
        Ok(())
    }

    pub async fn handle_stream_message(
        &mut self,
        message: &StreamMessage,
        redis_connection: &mut MultiplexedConnection,
    ) -> Result<Option<StreamEvent>, Box<dyn Error + Send + Sync>> {
        let (new_assets, needs_clear, needs_rates) = match message {
            StreamMessage::SubscribePrices(msg) => {
                let assets_set: HashSet<AssetId> = msg.assets.iter().cloned().collect();
                (assets_set, true, true)
            }
            StreamMessage::AddPrices(msg) => {
                let assets_set: HashSet<AssetId> = msg.assets.iter().cloned().collect();
                (assets_set, false, false)
            }
            StreamMessage::UnsubscribePrices(msg) => {
                let removed_channels: Vec<String> = msg.assets.iter().map(|id| id.to_string()).collect();
                for asset in &msg.assets {
                    self.assets.remove(asset);
                }
                let payload = self.fetch_payload(false).await?;
                redis_connection.unsubscribe(removed_channels).await?;
                return Ok(Some(StreamEvent::Prices(payload)));
            }
            StreamMessage::SubscribeRealtimePrices(_) | StreamMessage::UnsubscribeRealtimePrices(_) => return Ok(None),
        };

        if needs_clear {
            let old_channels = self.get_channel_ids();
            self.assets.clear();
            self.prices_to_publish.clear();
            if !old_channels.is_empty() {
                redis_connection.unsubscribe(old_channels).await?;
            }
        }
        self.assets.extend(new_assets);

        let _ = self.price_client.lock().await.track_observed_assets(&self.assets.ids()).await;

        let payload = self.fetch_payload(needs_rates).await?;
        redis_connection.subscribe(self.get_channel_ids()).await?;
        Ok(Some(StreamEvent::Prices(payload)))
    }

    async fn fetch_payload(&self, fetch_rates: bool) -> Result<WebSocketPricePayload, Box<dyn Error + Send + Sync>> {
        let client = self.price_client.lock().await;
        let prices = client
            .get_cache_prices(self.assets.ids())
            .await?
            .into_iter()
            .map(|x| x.as_asset_price_primitive())
            .collect();
        let rates = if fetch_rates { client.get_cache_fiat_rates().await? } else { vec![] };
        Ok(WebSocketPricePayload { prices, rates })
    }
}
