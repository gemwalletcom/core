pub mod client;

use crate::params::DeviceIdParam;
use crate::responders::{ApiError, ApiResponse};
pub use client::NotificationsClient;
use primitives::InAppNotification;
use rocket::{State, get, post, tokio::sync::Mutex};

// TODO: Remove once all clients migrate to /v1/devices/<device_id>/notifications
#[get("/notifications/<device_id>?<from_timestamp>")]
pub async fn get_notifications(
    device_id: DeviceIdParam,
    from_timestamp: Option<u64>,
    client: &State<Mutex<NotificationsClient>>,
) -> Result<ApiResponse<Vec<InAppNotification>>, ApiError> {
    Ok(client.lock().await.get_notifications(&device_id.0, from_timestamp)?.into())
}

// TODO: Remove once all clients migrate to /v1/devices/<device_id>/notifications/read
#[post("/notifications/<device_id>/read")]
pub async fn mark_notifications_read(device_id: DeviceIdParam, client: &State<Mutex<NotificationsClient>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.mark_all_as_read(&device_id.0)?.into())
}
