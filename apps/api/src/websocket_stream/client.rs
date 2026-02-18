use std::error::Error;
use std::sync::Arc;

use gem_tracing::info_with_fields;
use pricer::PriceClient;
use primitives::{AssetPrice, StreamEvent, StreamMessage, device_stream_channel};
use redis::PushInfo;
use redis::aio::MultiplexedConnection;
use rocket::futures::SinkExt;
use rocket::serde::json::serde_json;
use rocket::tokio::sync::Mutex;
use rocket_ws::Message;
use rocket_ws::stream::DuplexStream;

use super::price_handler::PriceHandler;
use crate::websocket::decode_push_message;

pub struct StreamObserverConfig {
    pub redis_url: String,
}

pub struct StreamObserverClient {
    device_id: String,
    device_channel: String,
    price_handler: PriceHandler,
}

impl StreamObserverClient {
    pub fn new(device_id: String, price_client: Arc<Mutex<PriceClient>>) -> Self {
        let device_channel = device_stream_channel(&device_id);
        Self {
            device_id,
            device_channel,
            price_handler: PriceHandler::new(price_client),
        }
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub async fn next_price_interval(&mut self) {
        self.price_handler.next_interval().await;
    }

    pub fn take_prices(&mut self) -> Vec<AssetPrice> {
        self.price_handler.take_prices()
    }

    pub async fn subscribe_device_channel(&self, redis_connection: &mut MultiplexedConnection) -> Result<(), Box<dyn Error + Send + Sync>> {
        redis_connection.subscribe(&self.device_channel).await?;
        Ok(())
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
            Message::Close(_) => {
                info_with_fields!("websocket client closed connection gracefully", status = "ok");
                Ok(())
            }
            Message::Pong(_) | Message::Frame(_) => Ok(()),
        }
    }

    async fn handle_message_payload(&mut self, data: Vec<u8>, redis_connection: &mut MultiplexedConnection, stream: &mut DuplexStream) -> Result<(), Box<dyn Error + Send + Sync>> {
        let message = serde_json::from_slice::<StreamMessage>(&data)?;
        if let Some(event) = self.price_handler.handle_stream_message(&message, redis_connection).await? {
            self.send_event(stream, event).await?;
        }
        Ok(())
    }

    pub fn handle_redis_message(&mut self, message: &PushInfo) -> Result<Option<StreamEvent>, Box<dyn Error + Send + Sync>> {
        let Some((channel, value)) = decode_push_message(message) else {
            return Ok(None);
        };

        if channel == self.device_channel {
            Ok(Some(serde_json::from_slice::<StreamEvent>(value)?))
        } else {
            self.price_handler.handle_price_message(value)?;
            Ok(None)
        }
    }

    pub async fn send_event(&self, stream: &mut DuplexStream, event: StreamEvent) -> Result<(), Box<dyn Error + Send + Sync>> {
        let text = serde_json::to_string(&event)?;
        Ok(stream.send(Message::Text(text)).await?)
    }
}
