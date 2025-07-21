mod client;
pub use client::MetricsClient;

use rocket::{get, tokio::sync::Mutex, State};

#[get("/")]
pub async fn get_metrics(client: &State<Mutex<MetricsClient>>) -> String {
    client.lock().await.get()
}
