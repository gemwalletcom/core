pub mod client;
use crate::responders::{ApiError, ApiResponse};
pub use client::DevicesClient;
use primitives::device::Device;
use rocket::{State, delete, get, post, put, tokio::sync::Mutex};

#[post("/devices", format = "json", data = "<device>")]
pub async fn add_device(device: rocket::serde::json::Json<Device>, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.add_device(device.0)?.into())
}

#[get("/devices/<device_id>")]
pub async fn get_device(device_id: &str, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.get_device(device_id)?.into())
}

#[put("/devices/<device_id>", format = "json", data = "<device>")]
pub async fn update_device(
    device: rocket::serde::json::Json<Device>,
    #[allow(unused)] device_id: &str,
    client: &State<Mutex<DevicesClient>>,
) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.update_device(device.0)?.into())
}

#[post("/devices/<device_id>/push-notification")]
pub async fn send_push_notification_device(device_id: &str, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(ApiResponse::from(
        client.lock().await.send_push_notification_device(device_id).await.map_err(ApiError::from)?,
    ))
}

#[delete("/devices/<device_id>")]
pub async fn delete_device(device_id: &str, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_device(device_id)?.into())
}
