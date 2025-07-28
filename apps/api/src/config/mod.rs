pub mod client;
pub use client::ConfigClient;

use primitives::config::ConfigResponse;
use rocket::{get, serde::json::Json, tokio::sync::Mutex, State};

#[get("/config")]
pub async fn get_config(config_client: &State<Mutex<ConfigClient>>) -> Json<ConfigResponse> {
    let config: ConfigResponse = config_client.lock().await.get_config().unwrap();
    Json(config)
}
