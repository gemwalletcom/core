mod client;
use crate::responders::ApiError;
pub use client::MetricsClient;
use rocket::{State, get, response::content::RawText, tokio::sync::Mutex};

#[get("/")]
pub async fn get_metrics(client: &State<Mutex<MetricsClient>>) -> Result<RawText<String>, ApiError> {
    Ok(RawText(client.lock().await.get()))
}
