mod client;
use crate::responders::ApiError;
pub use client::MetricsClient;
use rocket::{get, response::content::RawText, tokio::sync::Mutex, State};

#[get("/")]
pub async fn get_metrics(client: &State<Mutex<MetricsClient>>) -> Result<RawText<String>, ApiError> {
    Ok(RawText(client.lock().await.get()))
}
