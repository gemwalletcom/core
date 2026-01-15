pub mod client;

use crate::params::DeviceIdParam;
use crate::responders::{ApiError, ApiResponse};
pub use client::NotificationsClient;
use primitives::Notification;
use rocket::{State, get, post, tokio::sync::Mutex};

#[get("/notifications/<device_id>")]
pub async fn get_notifications(
    device_id: DeviceIdParam,
    client: &State<Mutex<NotificationsClient>>,
) -> Result<ApiResponse<Vec<Notification>>, ApiError> {
    Ok(client.lock().await.get_notifications(&device_id.0)?.into())
}

#[post("/notifications/<device_id>/read")]
pub async fn mark_notifications_read(
    device_id: DeviceIdParam,
    client: &State<Mutex<NotificationsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.mark_all_as_read(&device_id.0)?.into())
}
