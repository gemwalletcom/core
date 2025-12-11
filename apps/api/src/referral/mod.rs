mod client;

pub use client::ReferralClient;

use crate::responders::{ApiError, ApiResponse};
use primitives::{Referral, ReferralCodeRequest, ReferralEventItem};
use rocket::serde::json::Json;
use rocket::{State, get, post};
use tokio::sync::Mutex;

#[get("/referral/<address>")]
pub async fn get_referral(address: &str, client: &State<Mutex<ReferralClient>>) -> Result<ApiResponse<Referral>, ApiError> {
    Ok(client.lock().await.get_referral(address)?.into())
}

#[get("/referral/<address>/events")]
pub async fn get_referral_events(address: &str, client: &State<Mutex<ReferralClient>>) -> Result<ApiResponse<Vec<ReferralEventItem>>, ApiError> {
    Ok(client.lock().await.get_referral_events(address)?.into())
}

#[post("/referral/create", format = "json", data = "<request>")]
pub async fn create_referral(request: Json<ReferralCodeRequest>, client: &State<Mutex<ReferralClient>>) -> Result<ApiResponse<Referral>, ApiError> {
    Ok(client.lock().await.create_referral(&request.0)?.into())
}

#[post("/referral/use", format = "json", data = "<request>")]
pub async fn use_referral_code(request: Json<ReferralCodeRequest>, client: &State<Mutex<ReferralClient>>) -> Result<ApiResponse<bool>, ApiError> {
    client.lock().await.use_referral_code(&request.0)?;
    Ok(true.into())
}
