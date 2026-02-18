use gem_tracing::info_with_fields;
use primitives::WebSocketPricePayload;
use rocket::futures::StreamExt;
use rocket_ws::stream::DuplexStream;

use super::client::PriceObserverClient;

pub async fn new_stream(redis_url: &str, observer: &mut PriceObserverClient, stream: DuplexStream) {
    let Ok((mut stream, mut redis_connection, mut rx)) = crate::websocket::setup_ws_resources(redis_url, stream).await else {
        info_with_fields!("websocket failed to setup redis connection", status = "error");
        return;
    };

    loop {
        tokio::select! {
            biased;
            _ = observer.next_interval() => {
                let prices = observer.take_prices();
                if prices.is_empty() {
                    continue;
                }

                let payload = WebSocketPricePayload { prices, rates: vec![] };
                if observer.send_payload(&mut stream, payload).await.is_err() {
                    break;
                }
            }
            Some(message) = rx.recv() => {
                if let Err(e) = observer.handle_redis_message(&message) {
                    info_with_fields!("websocket redis message handler error", message = format!("{e:?}"), status = "error");
                }
            }
            message = stream.next() => {
                match message {
                    Some(Ok(message)) => {
                        if let Err(e) = observer.handle_ws_message(message, &mut redis_connection, &mut stream).await {
                            info_with_fields!("websocket message handler error", message = format!("{e:?}"), status = "error");
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
}
