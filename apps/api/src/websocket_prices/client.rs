use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use futures_util::SinkExt;
use pricer::PriceClient;
use primitives::asset_id::AssetIdHashSetExt;
use primitives::{AssetId, AssetPrice, AssetPriceInfo};
use primitives::{WebSocketPriceAction, WebSocketPriceActionType, WebSocketPricePayload};
use redis::aio::MultiplexedConnection;
use redis::PushInfo;
use redis::PushKind;
use rocket::serde::json::serde_json;
use rocket::tokio::sync::Mutex;
use rocket_ws::result::Error as WsError;
use rocket_ws::stream::DuplexStream;
use rocket_ws::Message;

pub struct PriceObserverConfig {
    pub redis_url: String,
}

pub struct PriceObserverClient {
    pub price_client: Arc<Mutex<PriceClient>>,
    pub assets: HashSet<AssetId>,
    prices_to_publish: std::collections::HashMap<String, AssetPrice>,
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

    pub fn get_asset_ids(&self) -> Vec<String> {
        self.assets.iter().map(|id| id.to_string()).collect()
    }

    pub fn add_price_to_publish(&mut self, price: AssetPrice) {
        self.prices_to_publish.insert(price.asset_id.clone(), price);
    }

    pub fn clear_prices_to_publish(&mut self) {
        self.prices_to_publish.clear();
    }

    pub fn get_prices_to_publish(&self) -> Vec<AssetPrice> {
        self.prices_to_publish.values().cloned().collect()
    }

    pub async fn fetch_payload_data(&mut self, fetch_rates: bool) -> Result<WebSocketPricePayload, Box<dyn Error + Send + Sync>> {
        let price_client_clone_prices = Arc::clone(&self.price_client);
        let assets_clone_prices = self.assets.clone();
        let prices = price_client_clone_prices
            .lock()
            .await
            .get_cache_prices(assets_clone_prices.ids())
            .await?
            .into_iter()
            .map(|x| x.as_asset_price_primitive())
            .collect();

        if fetch_rates {
            let rates = self.price_client.lock().await.get_cache_fiat_rates().await?;
            Ok(WebSocketPricePayload { prices, rates })
        } else {
            Ok(WebSocketPricePayload { prices, rates: vec![] })
        }
    }

    /// Serializes and sends the payload over the WebSocket stream
    pub async fn build_and_send_payload(
        &mut self,
        stream: &mut DuplexStream, // Pass stream mutably
        payload: WebSocketPricePayload,
    ) -> Result<(), WsError> {
        let data = match serde_json::to_vec(&payload) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to serialize price payload: {}", e);
                return Ok(()); // Log error, don't terminate
            }
        };
        let message = Message::Binary(data);
        stream.send(message).await // Use the passed-in stream
    }

    pub async fn handle_ws_message(
        &mut self,
        message: rocket_ws::Message,
        redis_connection: &mut MultiplexedConnection,
        stream: &mut DuplexStream,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match message {
            Message::Binary(data) => Ok(self.handle_message_payload(data, redis_connection, stream).await?),
            Message::Text(text) => Ok(self.handle_message_payload(text.into_bytes(), redis_connection, stream).await?),
            Message::Close(_) => {
                println!("Client closed connection gracefully");
                Ok(())
            }
            Message::Frame(_) | Message::Ping(_) | Message::Pong(_) => {
                eprintln!("WebSocket read error: Unsupported message type");
                Ok(())
            }
        }
    }

    pub async fn handle_message_payload(
        &mut self,
        data: Vec<u8>,
        redis_connection: &mut MultiplexedConnection,
        stream: &mut DuplexStream,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let action = serde_json::from_slice::<WebSocketPriceAction>(&data)?;
        match action.action {
            WebSocketPriceActionType::Subscribe => {
                self.assets.clear();
                self.assets.extend(action.assets.clone());
            }
            WebSocketPriceActionType::Add => {
                self.assets.extend(action.assets);
            }
        }

        let needs_rates = action.action == WebSocketPriceActionType::Subscribe;
        let payload = self.fetch_payload_data(needs_rates).await?;

        self.build_and_send_payload(stream, payload).await?;

        Ok(redis_connection.subscribe(self.get_asset_ids()).await?)
    }

    pub fn handle_redis_message(&mut self, message: &PushInfo) -> Result<(), String> {
        match (message.kind.clone(), message.data.last()) {
            (PushKind::Message, Some(redis::Value::BulkString(value))) => {
                let asset_price_info = serde_json::from_slice::<AssetPriceInfo>(value).map_err(|e| format!("Failed to deserialize AssetPrice: {}", e))?;
                let asset_price = asset_price_info.as_asset_price_primitive();
                self.add_price_to_publish(asset_price);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
