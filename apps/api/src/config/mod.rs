pub mod client;
use crate::responders::{ApiError, ApiResponse};
pub use client::ConfigClient;
use primitives::config::ConfigResponse;
use rocket::{State, get, tokio::sync::Mutex};

#[get("/config")]
pub async fn get_config(config_client: &State<Mutex<ConfigClient>>) -> Result<ApiResponse<ConfigResponse>, ApiError> {
    Ok(config_client.lock().await.get_config()?.into())
}
