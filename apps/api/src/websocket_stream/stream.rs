use gem_tracing::info_with_fields;
use primitives::{StreamEvent, WebSocketPricePayload};
use rocket::futures::StreamExt;
use rocket_ws::stream::DuplexStream;

use super::client::StreamObserverClient;

pub async fn new_stream(redis_url: &str, observer: &mut StreamObserverClient, stream: DuplexStream) {
    let Ok((mut stream, mut redis_connection, mut rx)) = crate::websocket::setup_ws_resources(redis_url, stream).await else {
        info_with_fields!("websocket failed to setup redis connection", status = "error");
        return;
    };

    info_with_fields!("websocket device stream connected", device_id = observer.device_id(), status = "ok");

    if let Err(e) = observer.subscribe_device_channel(&mut redis_connection).await {
        info_with_fields!("websocket failed to subscribe device channel", message = format!("{e:?}"), status = "error");
        return;
    }

    loop {
        tokio::select! {
            biased;
            _ = observer.next_price_interval() => {
                let prices = observer.take_prices();
                if prices.is_empty() {
                    continue;
                }

                let payload = WebSocketPricePayload { prices, rates: vec![] };
                match observer.send_event(&mut stream, StreamEvent::Prices(payload)).await {
                    Ok(_) => {
                        info_with_fields!("websocket tick notified prices", status = "ok");
                    }
                    Err(e) => {
                        info_with_fields!("websocket send error on tick", message = format!("{e:?}"), status = "error");
                        break;
                    }
                }
            }
            Some(message) = rx.recv() => {
                match observer.handle_redis_message(&message) {
                    Ok(Some(event)) => {
                        if let Err(e) = observer.send_event(&mut stream, event).await {
                            info_with_fields!("websocket send event error", message = format!("{e:?}"), status = "error");
                            break;
                        }
                    }
                    Ok(None) => { }
                    Err(e) => {
                        info_with_fields!("websocket redis message handler error", message = format!("{e:?}"), status = "error");
                    }
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
    info_with_fields!("websocket device stream disconnected", device_id = observer.device_id(), status = "ok");
}
