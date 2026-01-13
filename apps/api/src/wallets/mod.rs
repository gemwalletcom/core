pub mod client;
use crate::params::DeviceIdParam;
use crate::responders::{ApiError, ApiResponse};
pub use client::WalletsClient;
use primitives::{WalletSubscription, WalletSubscriptionChains};
use rocket::{State, delete, get, post, serde::json::Json, tokio::sync::Mutex};

#[get("/subscriptions/<device_id>")]
pub async fn get_subscriptions(device_id: DeviceIdParam, client: &State<Mutex<WalletsClient>>) -> Result<ApiResponse<Vec<WalletSubscriptionChains>>, ApiError> {
    Ok(client.lock().await.get_subscriptions(&device_id.0).await?.into())
}

#[post("/subscriptions/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn add_subscriptions(
    device_id: DeviceIdParam,
    subscriptions: Json<Vec<WalletSubscription>>,
    client: &State<Mutex<WalletsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.add_subscriptions(&device_id.0, subscriptions.0).await?.into())
}

#[delete("/subscriptions/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn delete_subscriptions(
    device_id: DeviceIdParam,
    subscriptions: Json<Vec<WalletSubscription>>,
    client: &State<Mutex<WalletsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_subscriptions(&device_id.0, subscriptions.0).await?.into())
}
