pub mod client;
use crate::responders::{ApiError, ApiResponse};
pub use client::WalletsClient;
use primitives::Subscription;
use rocket::{State, delete, get, post, serde::json::Json, tokio::sync::Mutex};

#[get("/subscriptions/<device_id>/<wallet_id>")]
pub async fn get_subscriptions(device_id: i32, wallet_id: String, client: &State<Mutex<WalletsClient>>) -> Result<ApiResponse<Vec<Subscription>>, ApiError> {
    Ok(client.lock().await.get_wallet_subscriptions(device_id, &wallet_id).await?.into())
}

#[post("/subscriptions/<device_id>/<wallet_id>", format = "json", data = "<subscriptions>")]
pub async fn add_subscriptions(
    device_id: i32,
    wallet_id: String,
    subscriptions: Json<Vec<Subscription>>,
    client: &State<Mutex<WalletsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.add_wallet_subscriptions(device_id, &wallet_id, subscriptions.0).await?.into())
}

#[delete("/subscriptions/<device_id>/<wallet_id>", format = "json", data = "<subscriptions>")]
pub async fn delete_subscriptions(
    device_id: i32,
    wallet_id: String,
    subscriptions: Json<Vec<Subscription>>,
    client: &State<Mutex<WalletsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_wallet_subscriptions(device_id, &wallet_id, subscriptions.0).await?.into())
}
