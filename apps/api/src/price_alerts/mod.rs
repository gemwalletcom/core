use crate::responders::{ApiError, ApiResponse};
pub use pricer::PriceAlertClient;
use primitives::PriceAlerts;
use rocket::{delete, get, post, serde::json::Json, tokio::sync::Mutex, State};

#[get("/price_alerts/<device_id>")]
pub async fn get_price_alerts(device_id: &str, client: &State<Mutex<PriceAlertClient>>) -> Result<ApiResponse<PriceAlerts>, ApiError> {
    Ok(client.lock().await.get_price_alerts(device_id).await?.into())
}

#[post("/price_alerts/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn add_price_alerts(
    device_id: &str,
    subscriptions: Json<PriceAlerts>,
    client: &State<Mutex<PriceAlertClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.add_price_alerts(device_id, subscriptions.0).await?.into())
}

#[delete("/price_alerts/<device_id>", format = "json", data = "<subscriptions>")]
pub async fn delete_price_alerts(
    device_id: &str,
    subscriptions: Json<PriceAlerts>,
    client: &State<Mutex<PriceAlertClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_price_alerts(device_id, subscriptions.0).await?.into())
}
