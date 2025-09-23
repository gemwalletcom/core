pub mod client;
use crate::responders::{ApiError, ApiResponse};
pub use client::SubscriptionsClient;
use primitives::Subscription;
use rocket::{State, delete, get, post, serde::json::Json, tokio::sync::Mutex};

#[get("/subscriptions/<device_id>")]
pub async fn get_subscriptions(device_id: &str, client: &State<Mutex<SubscriptionsClient>>) -> Result<ApiResponse<Vec<Subscription>>, ApiError> {
    Ok(client.lock().await.get_subscriptions_by_device_id(device_id).await?.into())
}

#[delete("/subscriptions/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn delete_subscriptions(
    subscriptions: Json<Vec<Subscription>>,
    device_id: &str,
    client: &State<Mutex<SubscriptionsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_subscriptions(device_id, subscriptions.0).await?.into())
}

#[post("/subscriptions/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn add_subscriptions(
    subscriptions: Json<Vec<Subscription>>,
    device_id: &str,
    client: &State<Mutex<SubscriptionsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.add_subscriptions(device_id, subscriptions.0).await?.into())
}
