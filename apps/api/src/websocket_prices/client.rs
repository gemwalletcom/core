use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use pricer::PriceClient;
use primitives::{AssetId, AssetPrice, AssetPriceInfo, WebSocketPriceAction, WebSocketPriceActionType, WebSocketPricePayload, asset::AssetHashSetExt};
use redis::PushInfo;
use redis::aio::MultiplexedConnection;
use rocket::futures::SinkExt;
use rocket::serde::json::serde_json;
use rocket::tokio::sync::Mutex;
use rocket_ws::Message;
use rocket_ws::stream::DuplexStream;

use crate::websocket::decode_push_message;

pub struct PriceObserverConfig {
    pub redis_url: String,
}

pub struct PriceObserverClient {
    price_client: Arc<Mutex<PriceClient>>,
    assets: HashSet<AssetId>,
    prices_to_publish: HashMap<String, AssetPrice>,
    interval: rocket::tokio::time::Interval,
}

impl PriceObserverClient {
    pub fn new(price_client: Arc<Mutex<PriceClient>>) -> Self {
        PriceObserverClient {
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

    pub async fn handle_ws_message(
        &mut self,
        message: Message,
        redis_connection: &mut MultiplexedConnection,
        stream: &mut DuplexStream,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match message {
            Message::Binary(data) => self.handle_message_payload(data, redis_connection, stream).await,
            Message::Text(text) => self.handle_message_payload(text.into_bytes(), redis_connection, stream).await.or(Ok(())),
            Message::Ping(data) => Ok(stream.send(Message::Pong(data)).await?),
            Message::Close(_) => Ok(()),
            Message::Pong(_) | Message::Frame(_) => Ok(()),
        }
    }

    async fn handle_message_payload(&mut self, data: Vec<u8>, redis_connection: &mut MultiplexedConnection, stream: &mut DuplexStream) -> Result<(), Box<dyn Error + Send + Sync>> {
        let action = serde_json::from_slice::<WebSocketPriceAction>(&data)?;
        let new_assets: HashSet<AssetId> = action.assets.iter().cloned().collect();

        let needs_rates = match action.action {
            WebSocketPriceActionType::Subscribe => {
                let old_channels = self.get_channel_ids();
                self.assets.clear();
                self.prices_to_publish.clear();
                if !old_channels.is_empty() {
                    redis_connection.unsubscribe(old_channels).await?;
                }
                self.assets.extend(new_assets);
                true
            }
            WebSocketPriceActionType::Add => {
                self.assets.extend(new_assets);
                false
            }
        };

        let _ = self.price_client.lock().await.track_observed_assets(&self.assets.ids()).await;

        let payload = self.fetch_payload(needs_rates).await?;
        self.send_payload(stream, payload).await?;

        redis_connection.subscribe(self.get_channel_ids()).await?;
        Ok(())
    }

    pub async fn send_payload(&self, stream: &mut DuplexStream, payload: WebSocketPricePayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        let text = serde_json::to_string(&payload)?;
        Ok(stream.send(Message::Text(text)).await?)
    }

    pub fn handle_redis_message(&mut self, message: &PushInfo) -> Result<(), Box<dyn Error + Send + Sync>> {
        let Some((_, value)) = decode_push_message(message) else {
            return Ok(());
        };
        let info = serde_json::from_slice::<AssetPriceInfo>(value)?;
        self.prices_to_publish.insert(info.asset_id.to_string(), info.as_asset_price_primitive());
        Ok(())
    }
}
