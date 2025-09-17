use rocket::futures::StreamExt;
use primitives::websocket::WebSocketPricePayload;
use redis::aio::MultiplexedConnection;
use rocket_ws::stream::DuplexStream;
use tokio::sync::mpsc::UnboundedReceiver;

use super::client::PriceObserverClient;

pub struct Stream;

impl Stream {
    pub async fn setup_ws_resources(redis_url: &str, stream: DuplexStream) -> (DuplexStream, MultiplexedConnection, UnboundedReceiver<redis::PushInfo>) {
        let stream = stream;
        let client = redis::Client::open(redis_url).unwrap();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let config = redis::AsyncConnectionConfig::new().set_push_sender(tx);
        let redis_connection = client.get_multiplexed_async_connection_with_config(&config).await.unwrap();
        (stream, redis_connection, rx)
    }

    pub async fn new_stream(redis_url: &str, observer: &mut PriceObserverClient, stream: DuplexStream) {
        let (mut stream, mut redis_connection, mut rx) = Self::setup_ws_resources(redis_url, stream).await;

        println!("WebSocket client connected");

        loop {
            tokio::select! {
                biased;
                // interval tick
                _ = observer.next_interval() => {
                    let prices = observer.get_prices_to_publish();

                    //println!("Tick: assets: {:?}, prices: {}", observer.get_asset_ids().len(), prices.len());

                    if prices.is_empty() {
                        continue;
                    }

                    let payload = WebSocketPricePayload { prices, rates: vec![] };
                    match observer.build_and_send_payload(&mut stream, payload.clone()).await {
                        Ok(_) => {
                            eprintln!("Tick: notified prices: {}", payload.prices.len());
                        }
                        Err(e) => {
                            eprintln!("WebSocket send error on tick: {e:?}");
                            break;
                        }
                    }

                    observer.clear_prices_to_publish();
                }
                // new redis message pub/sub
                Some(message) = rx.recv() => {
                    match observer.handle_redis_message(&message) {
                        Ok(_) => { }
                        Err(e) => {
                            eprintln!("WebSocket Redis message handler error: {e:?}");
                        }
                    }
                }
                // new websocket message
                Some(message) = stream.next() => {
                    match message {
                        Ok(message) => {
                            match observer.handle_ws_message(message.clone(), &mut redis_connection, &mut stream).await {
                                Ok(_) => { }
                                Err(e) => {
                                    eprintln!("WebSocket message handler error: {e:?}");
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("WebSocket stream error: {e:?}");
                        }
                    }
                }
            }
        }
        println!("WebSocket client disconnected (loop in stream.rs)");
    }
}
