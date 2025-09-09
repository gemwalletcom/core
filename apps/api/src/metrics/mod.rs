mod client;
use crate::responders::{ApiError, ApiResponse};
pub use client::MetricsClient;
use rocket::{get, tokio::sync::Mutex, State};

#[get("/")]
pub async fn get_metrics(client: &State<Mutex<MetricsClient>>) -> Result<ApiResponse<String>, ApiError> {
    Ok(client.lock().await.get().into())
}
