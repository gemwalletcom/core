use std::sync::Arc;

use pricer::PriceClient;
use rocket::State;
use rocket::tokio::sync::Mutex;
use rocket_ws::{Channel, WebSocket};

use crate::devices::guard::AuthenticatedDevice;

mod client;
mod stream;

pub use client::{StreamObserverClient, StreamObserverConfig};
use stream::Stream;

#[rocket::get("/stream")]
pub async fn ws_stream(
    ws: WebSocket,
    _auth: AuthenticatedDevice,
    price_client: &State<Arc<Mutex<PriceClient>>>,
    config: &State<Arc<Mutex<StreamObserverConfig>>>,
) -> Channel<'static> {
    let price_client = price_client.inner().clone();
    let redis_url = config.lock().await.redis_url.clone();

    ws.channel(move |stream| {
        Box::pin(async move {
            let mut observer = StreamObserverClient::new(price_client.clone());
            Stream::new_stream(&redis_url, &mut observer, stream).await;
            Ok::<(), rocket_ws::result::Error>(())
        })
    })
}
