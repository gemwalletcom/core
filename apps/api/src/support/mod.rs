pub mod client;

use crate::responders::{ApiError, ApiResponseNew};
pub use client::SupportClient;
use primitives::SupportDevice;
use rocket::{post, serde::json::Json, tokio::sync::Mutex, State};

#[post("/support/add_device", format = "json", data = "<request>")]
pub async fn add_device(request: Json<SupportDevice>, client: &State<Mutex<SupportClient>>) -> Result<ApiResponseNew<SupportDevice>, ApiError> {
    let support_device = client.lock().await.add_support_device(&request.support_id, &request.device_id)?;
    Ok(ApiResponseNew::from(support_device))
}
