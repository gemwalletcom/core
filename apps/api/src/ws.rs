use futures_util::{SinkExt, StreamExt};
use primitives::AssetPrice;
use rocket::tokio::select;
use rocket::tokio::time::Duration;
use rocket_ws::Channel;
use rocket_ws::Message;
use rocket_ws::WebSocket;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AssetList(pub Vec<String>);

#[rocket::get("/prices")]
pub async fn ws_prices(ws: WebSocket) -> Channel<'static> {
    ws.channel(move |mut stream| {
        Box::pin(async move {
            let mut assets: Vec<String> = Vec::new(); // Hold current subscribed assets
            let mut interval = rocket::tokio::time::interval(Duration::from_secs(1));

            println!("WebSocket client connected.");

            loop {
                select! {
                    // Handle incoming messages from the client
                    msg = stream.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                println!("Received message: {}", text);
                                match serde_json::from_str::<AssetList>(&text) {
                                    Ok(new_list) => {
                                        if new_list.0.is_empty() {
                                            println!("Received empty asset list, clearing subscription.");
                                            assets.clear();
                                            // Optionally send confirmation back
                                             let _ = stream.send(Message::Text("Subscription cleared.".to_string())).await;
                                        } else {
                                            println!("Updating asset subscription to: {:?}", new_list.0);
                                            assets = new_list.0;
                                             // Optionally send confirmation back
                                             let _ = stream.send(Message::Text(format!("Subscription updated to: {:?}", assets))).await;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Asset list JSON parsing error: {}", e);
                                        let _ = stream.send(Message::Text("Invalid asset list format.".to_string())).await;
                                    }
                                }
                            }
                            Some(Ok(other)) => {
                                eprintln!("Received unexpected message type: {:?}, ignoring.", other);
                                // Optionally send an error message back
                            }
                            Some(Err(e)) => {
                                eprintln!("WebSocket receive error: {}, closing connection.", e);
                                break; // Exit loop on receive error
                            }
                            None => {
                                println!("WebSocket connection closed by client.");
                                break; // Exit loop when client disconnects
                            }
                        }
                    }

                    // Tick the interval timer to send price updates
                    _ = interval.tick() => {
                        if assets.is_empty() {
                            // println!("No assets subscribed, skipping price update.");
                            continue; // Skip sending if no assets are subscribed
                        }

                        let mut updates = Vec::new();
                        for asset in &assets {
                            let price = match asset.as_str() {
                                "bitcoin" => 65000.0 + rand::random::<f64>() * 100.0,
                                "ethereum" => 3200.0 + rand::random::<f64>() * 10.0,
                                _ => 0.0,
                            };
                            updates.push(AssetPrice {asset_id: asset.clone(), price,  price_change_percentage_24h: rand::random::<f64>() * 100.0 });
                        }

                        if !updates.is_empty() {
                           let payload = serde_json::to_string(&updates).unwrap();
                            // println!("Sending update: {}", payload);
                            if stream.send(Message::Text(payload)).await.is_err() {
                                eprintln!("WebSocket send error, closing connection.");
                                break; // Exit loop on send error
                            }
                        }
                    }
                }
            }

            println!("WebSocket handler loop finished.");
            Ok(())
        })
    })
}
