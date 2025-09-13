mod client;
use crate::responders::ApiError;
pub use client::MetricsClient;
use rocket::{get, tokio::sync::Mutex, State, response::content::RawText};

#[get("/")]
pub async fn get_metrics(client: &State<Mutex<MetricsClient>>) -> Result<RawText<String>, ApiError> {
    Ok(RawText(client.lock().await.get()))
}
