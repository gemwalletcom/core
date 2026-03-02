pub mod client;
use crate::params::DeviceIdParam;
use crate::responders::{ApiError, ApiResponse};
use client::WalletSubscriptionInput;
pub use client::WalletsClient;
use primitives::WalletSubscriptionChains;
use rocket::{State, delete, get, post, serde::json::Json, tokio::sync::Mutex};

// TODO: Remove once all clients migrate to /v1/devices/<device_id>/subscriptions
#[get("/subscriptions/<device_id>")]
pub async fn get_subscriptions(device_id: DeviceIdParam, client: &State<Mutex<WalletsClient>>) -> Result<ApiResponse<Vec<WalletSubscriptionChains>>, ApiError> {
    Ok(client.lock().await.get_subscriptions(&device_id.0).await?.into())
}

// TODO: Remove once all clients migrate to /v1/devices/<device_id>/subscriptions
#[post("/subscriptions/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn add_subscriptions(
    device_id: DeviceIdParam,
    subscriptions: Json<Vec<WalletSubscriptionInput>>,
    client: &State<Mutex<WalletsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    let wallet_subscriptions = subscriptions.0.into_iter().map(|x| x.into_wallet_subscription()).collect();
    Ok(client.lock().await.add_subscriptions(&device_id.0, wallet_subscriptions).await?.into())
}

// TODO: Remove once all clients migrate to /v2/devices/subscriptions
#[delete("/subscriptions/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn delete_subscriptions(
    device_id: DeviceIdParam,
    subscriptions: Json<Vec<WalletSubscriptionInput>>,
    client: &State<Mutex<WalletsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    let _ = (device_id, subscriptions, client);
    Ok(0.into())
}
