use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use pricer::PriceClient;
use primitives::asset_id::AssetIdHashSetExt;
use primitives::websocket::WebSocketPriceAction;
use primitives::{AssetId, WebSocketPricePayload};
use rocket::futures::{SinkExt, StreamExt};
use rocket::serde::json::serde_json;
use rocket::tokio::sync::Mutex;
use rocket::State;
use rocket_ws::result::Error as WsError;
use rocket_ws::{Channel, Message, WebSocket};

#[rocket::get("/prices")]
pub async fn ws_prices(ws: WebSocket, price_client: &State<Arc<Mutex<PriceClient>>>) -> Channel<'static> {
    let price_client = price_client.inner().clone();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            let mut assets: HashSet<AssetId> = HashSet::new();

            let mut update_rates = false;
            let price_client = price_client.clone();
            let mut interval = rocket::tokio::time::interval(Duration::from_secs(1));

            println!("WebSocket client connected");

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

                        let payload = WebSocketPricePayload { prices, rates };
                        let data = serde_json::to_vec(&payload).unwrap();
                        let payload = Message::Binary(data);

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
