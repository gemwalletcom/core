use std::sync::Arc;

use pricer::PriceClient;
use rocket::State;
use rocket::http::Status;
use rocket::tokio::sync::Mutex;
use rocket_ws::{Channel, WebSocket};

mod client;
mod stream;

pub use client::PriceObserverConfig;

#[rocket::get("/prices")]
pub async fn ws_prices(ws: WebSocket, price_client: &State<Arc<Mutex<PriceClient>>>, config: &State<Arc<PriceObserverConfig>>) -> Channel<'static> {
    let price_client = price_client.inner().clone();
    let redis_url = config.redis_url.clone();

    ws.channel(move |ws_stream| {
        Box::pin(async move {
            let mut observer = client::PriceObserverClient::new(price_client);
            stream::new_stream(&redis_url, &mut observer, ws_stream).await;
            Ok::<(), rocket_ws::result::Error>(())
        })
    })
}

#[rocket::get("/health")]
pub fn ws_health() -> Status {
    Status::Ok
}
