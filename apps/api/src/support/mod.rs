pub mod client;

use crate::devices::guard::AuthenticatedDevice;
use crate::responders::{ApiError, ApiResponse};
pub use client::SupportClient;
use primitives::{NewSupportDevice, SupportDevice, SupportDeviceRequest};
use rocket::{State, get, post, serde::json::Json, tokio::sync::Mutex};

#[post("/support/add_device", format = "json", data = "<request>")]
pub async fn add_device_legacy(request: Json<NewSupportDevice>, client: &State<Mutex<SupportClient>>) -> Result<ApiResponse<SupportDevice>, ApiError> {
    let support_device = client.lock().await.add_support_device(&request.support_device_id, &request.device_id)?;
    Ok(ApiResponse::from(support_device))
}

#[post("/devices/<_device_id>/support", format = "json", data = "<request>")]
pub async fn add_device(
    _device_id: &str,
    device: AuthenticatedDevice,
    request: Json<SupportDeviceRequest>,
    client: &State<Mutex<SupportClient>>,
) -> Result<ApiResponse<SupportDevice>, ApiError> {
    let support_device = client.lock().await.add_support_device(&request.support_device_id, &device.device_row.device_id)?;
    Ok(ApiResponse::from(support_device))
}

#[get("/support/<support_device_id>")]
pub async fn get_support_device(support_device_id: &str, client: &State<Mutex<SupportClient>>) -> Result<ApiResponse<SupportDevice>, ApiError> {
    let support_device = client.lock().await.get_support_device(support_device_id)?;
    Ok(ApiResponse::from(support_device))
}
