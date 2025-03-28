pub mod client;
pub use client::ConfigClient;
extern crate rocket;

use primitives::config::ConfigResponse;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/config")]
pub async fn get_config(config_client: &State<Mutex<ConfigClient>>) -> Json<ConfigResponse> {
    let config: ConfigResponse = config_client.lock().await.get_config().unwrap();
    Json(config)
}
