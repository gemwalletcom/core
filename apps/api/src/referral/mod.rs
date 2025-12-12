mod client;

pub use client::RewardsClient;

use crate::responders::{ApiError, ApiResponse};
use primitives::{Rewards, RewardsEventItem, RewardsReferralRequest};
use rocket::serde::json::Json;
use rocket::{State, get, post};
use tokio::sync::Mutex;

#[get("/rewards/<address>")]
pub async fn get_rewards(address: &str, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<Rewards>, ApiError> {
    Ok(client.lock().await.get_rewards(address)?.into())
}

#[get("/rewards/<address>/events")]
pub async fn get_rewards_events(address: &str, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<Vec<RewardsEventItem>>, ApiError> {
    Ok(client.lock().await.get_rewards_events(address)?.into())
}

#[post("/rewards/referrals/create", format = "json", data = "<request>")]
pub async fn create_referral(request: Json<RewardsReferralRequest>, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<Rewards>, ApiError> {
    Ok(client.lock().await.create_referral(&request.0).await?.into())
}

#[post("/rewards/referrals/use", format = "json", data = "<request>")]
pub async fn use_referral_code(request: Json<RewardsReferralRequest>, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<bool>, ApiError> {
    client.lock().await.use_referral_code(&request.0).await?;
    Ok(true.into())
}
