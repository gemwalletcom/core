use crate::params::DeviceIdParam;
use crate::responders::{ApiError, ApiResponse};
pub use pricer::PriceAlertClient;
use primitives::PriceAlerts;
use rocket::{State, delete, get, post, serde::json::Json, tokio::sync::Mutex};

// TODO: Remove once all clients migrate to /v1/devices/<device_id>/price_alerts
#[get("/price_alerts/<device_id>?<asset_id>")]
pub async fn get_price_alerts(device_id: DeviceIdParam, asset_id: Option<&str>, client: &State<Mutex<PriceAlertClient>>) -> Result<ApiResponse<PriceAlerts>, ApiError> {
    Ok(client.lock().await.get_price_alerts(&device_id.0, asset_id).await?.into())
}

// TODO: Remove once all clients migrate to /v1/devices/<device_id>/price_alerts
#[post("/price_alerts/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn add_price_alerts(device_id: DeviceIdParam, subscriptions: Json<PriceAlerts>, client: &State<Mutex<PriceAlertClient>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.add_price_alerts(&device_id.0, subscriptions.0).await?.into())
}

// TODO: Remove once all clients migrate to /v1/devices/<device_id>/price_alerts
#[delete("/price_alerts/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn delete_price_alerts(device_id: DeviceIdParam, subscriptions: Json<PriceAlerts>, client: &State<Mutex<PriceAlertClient>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_price_alerts(&device_id.0, subscriptions.0).await?.into())
}
