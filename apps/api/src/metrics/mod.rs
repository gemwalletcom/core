mod client;
pub use client::MetricsClient;

extern crate rocket;

use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/")]
pub async fn get_metrics(client: &State<Mutex<MetricsClient>>) -> String {
    client.lock().await.get()
}
