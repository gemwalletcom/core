use std::sync::Arc;

use rocket::State;
use rocket::tokio::sync::Mutex;
use rocket_ws::{Channel, WebSocket};

mod client;
mod stream;

pub use client::{PriceObserverClient, PriceObserverConfig};
use pricer::PriceClient;
use stream::Stream;

#[rocket::get("/prices")]
pub async fn ws_prices(ws: WebSocket, price_client: &State<Arc<Mutex<PriceClient>>>, config: &State<Arc<Mutex<PriceObserverConfig>>>) -> Channel<'static> {
    let price_client = price_client.inner().clone();
    let redis_url = config.lock().await.redis_url.clone();

    ws.channel(move |stream| {
        Box::pin(async move {
            let mut observer = PriceObserverClient::new(price_client.clone());
            Stream::new_stream(&redis_url, &mut observer, stream).await;
            Ok::<(), rocket_ws::result::Error>(())
        })
    })
}
