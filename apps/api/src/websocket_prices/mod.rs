use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use pricer::PriceClient;
use primitives::asset_id::AssetIdHashSetExt;
use primitives::websocket::WebSocketPriceAction;
use primitives::{AssetId, AssetPrice, FiatRate, WebSocketPricePayload};
use rocket::futures::{SinkExt, StreamExt};
use rocket::serde::json::serde_json;
use rocket::tokio::sync::Mutex;
use rocket::State;
use rocket_ws::result::Error as WsError;
use rocket_ws::{Channel, Message, WebSocket};

#[derive(Debug, Clone)]
pub enum MessageType {
    Text,
    Binary,
}

impl MessageType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "text" => MessageType::Text,
            "binary" => MessageType::Binary,
            _ => MessageType::Text,
        }
    }
}

pub struct MessagePayload {
    pub payload: Vec<u8>,
}

impl MessagePayload {
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }

    pub fn message(&self, message_type: MessageType) -> Message {
        match message_type {
            MessageType::Text => Message::Text(String::from_utf8_lossy(&self.payload).to_string()),
            MessageType::Binary => Message::Binary(self.payload.clone()),
        }
    }

    pub fn from(prices: Vec<AssetPrice>, rates: Vec<FiatRate>) -> MessagePayload {
        let payload = WebSocketPricePayload { prices, rates };
        MessagePayload::new(serde_json::to_vec(&payload).unwrap())
    }
}

#[rocket::get("/prices?<mode>")]
pub async fn ws_prices(ws: WebSocket, mode: Option<String>, price_client: &State<Arc<Mutex<PriceClient>>>) -> Channel<'static> {
    let price_client = price_client.inner().clone();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            let mut assets: HashSet<AssetId> = HashSet::new();

            let mode = MessageType::from_str(mode.as_ref().map_or("text", |v| v));
            let mut update_rates = false;
            let price_client = price_client.clone();
            let mut interval = rocket::tokio::time::interval(Duration::from_secs(1));

            println!("WebSocket client connected, mode: {:?}", mode);

            loop {
                tokio::select! {
                    message = stream.next() => {
                        match message {
                            Some(Ok(Message::Binary(data))) => {
                                println!("Received binary message: {:?}", data.len());

                                match serde_json::from_slice::<WebSocketPriceAction>(&data) {
                                    Ok(message) => {
                                        println!(
                                            "Got action={:?}, updating assets: {:?}",
                                            message.action,
                                            message.assets.len()
                                        );
                                        assets.extend(message.assets);
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to deserialize WebSocketPriceAction: {}", e);
                                    }
                                }
                            }
                            Some(Ok(Message::Text(text))) => {
                                println!("Received message: {}", text);

                                match serde_json::from_str::<WebSocketPriceAction>(&text) {
                                    Ok(action) => {
                                        assets.extend(action.assets.clone());

                                        //TODO: Send new assets to the client
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to deserialize asset list: {}", e);
                                    }
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                println!("Client requested close.");
                                break;
                            }
                            Some(Ok(other)) => {
                                 eprintln!("Received unexpected message type: {:?}, ignoring.", other);
                            }
                            Some(Err(e)) => {
                                eprintln!("WebSocket error: {}, closing connection.", e);
                                break;
                            }
                            None => {
                                println!("WebSocket connection closed by client.");
                                break;
                            }
                        }
                    }

                    _ = interval.tick() => {
                        if assets.is_empty() {
                            continue;
                        }

                        let rates = if !update_rates {
                            price_client.lock().await.get_cache_fiat_rates().await.unwrap()
                        } else {
                            vec![]
                        };

                        let prices = price_client.lock().await.get_cache_prices("USD", assets.ids()).await
                            .into_iter()
                            .flatten()
                            .map(|x| x.as_asset_price_primitive())
                            .collect::<Vec<_>>();

                        if prices.is_empty() {
                            continue;
                        }

                        let payload = MessagePayload::from(prices.clone(), rates).message(message_type.clone());

                        if stream.send(payload).await.is_err() {
                            eprintln!("WebSocket send error, closing connection.");
                            break;
                        }

                        update_rates = true
                    }
                }
            }
            println!("WebSocket handler loop finished.");
            Ok::<(), WsError>(())
        })
    })
}
