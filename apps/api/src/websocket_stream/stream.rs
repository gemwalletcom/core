use gem_tracing::info_with_fields;
use primitives::websocket::WebSocketPricePayload;
use redis::aio::MultiplexedConnection;
use rocket::futures::StreamExt;
use rocket_ws::stream::DuplexStream;
use tokio::sync::mpsc::UnboundedReceiver;

use super::client::StreamObserverClient;

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

    pub async fn new_stream(redis_url: &str, observer: &mut StreamObserverClient, stream: DuplexStream) {
        let (mut stream, mut redis_connection, mut rx) = Self::setup_ws_resources(redis_url, stream).await;

        info_with_fields!("websocket device stream connected", status = "ok");

        loop {
            tokio::select! {
                biased;
                _ = observer.next_interval() => {
                    let prices = observer.get_prices_to_publish();

                    if prices.is_empty() {
                        continue;
                    }

                    let payload = WebSocketPricePayload { prices, rates: vec![] };
                    match observer.build_and_send_payload(&mut stream, payload.clone()).await {
                        Ok(_) => {
                            info_with_fields!("websocket tick notified prices", count = payload.prices.len(), status = "ok");
                        }
                        Err(e) => {
                            info_with_fields!("websocket send error on tick", message = format!("{e:?}"), status = "error");
                            break;
                        }
                    }

                    observer.clear_prices_to_publish();
                }
                Some(message) = rx.recv() => {
                    match observer.handle_redis_message(&message) {
                        Ok(_) => { }
                        Err(e) => {
                            info_with_fields!("websocket redis message handler error", message = e, status = "error");
                        }
                    }
                }
                message = stream.next() => {
                    match message {
                        Some(Ok(message)) => {
                            match observer.handle_ws_message(message.clone(), &mut redis_connection, &mut stream).await {
                                Ok(_) => { }
                                Err(e) => {
                                    info_with_fields!("websocket message handler error", message = format!("{e:?}"), status = "error");
                                }
                            }
                        }
                        Some(Err(e)) => {
                            info_with_fields!("websocket stream error", message = format!("{e:?}"), status = "error");
                        }
                        None => {
                            break;
                        }
                    }
                }
            }
        }
        info_with_fields!("websocket device stream disconnected", status = "ok");
    }
}
