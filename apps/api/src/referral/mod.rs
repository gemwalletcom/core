mod client;
mod redemption_client;

pub use client::RewardsClient;
pub use redemption_client::RewardsRedemptionClient;

use crate::auth::Authenticated;
use crate::params::AddressParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::rewards::{RedemptionRequest, RedemptionResult};
use primitives::{ReferralCode, RewardEvent, Rewards};
use rocket::{State, get, post};
use tokio::sync::Mutex;

#[get("/rewards/<address>")]
pub async fn get_rewards(address: AddressParam, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<Rewards>, ApiError> {
    Ok(client.lock().await.get_rewards(&address.0)?.into())
}

#[get("/rewards/<address>/events")]
pub async fn get_rewards_events(address: AddressParam, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<Vec<RewardEvent>>, ApiError> {
    Ok(client.lock().await.get_rewards_events(&address.0)?.into())
}

#[post("/rewards/referrals/create", format = "json", data = "<request>")]
pub async fn create_referral(request: Authenticated<ReferralCode>, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<Rewards>, ApiError> {
    Ok(client.lock().await.create_referral(&request.auth.address, &request.data.code).await?.into())
}

#[allow(dead_code)]
#[post("/rewards/referrals/update", format = "json", data = "<request>")]
pub async fn update_referral(request: Authenticated<ReferralCode>, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<Rewards>, ApiError> {
    Ok(client.lock().await.change_username(&request.auth.address, &request.data.code)?.into())
}

#[post("/rewards/referrals/use", format = "json", data = "<request>")]
pub async fn use_referral_code(
    request: Authenticated<ReferralCode>,
    ip: std::net::IpAddr,
    client: &State<Mutex<RewardsClient>>,
) -> Result<ApiResponse<bool>, ApiError> {
    client
        .lock()
        .await
        .use_referral_code(&request.auth, &request.data.code, &ip.to_string())
        .await?;
    Ok(true.into())
}

#[post("/rewards/<address>/redeem", format = "json", data = "<request>")]
pub async fn redeem_rewards(
    address: AddressParam,
    request: Authenticated<RedemptionRequest>,
    client: &State<Mutex<RewardsRedemptionClient>>,
) -> Result<ApiResponse<RedemptionResult>, ApiError> {
    Ok(client.lock().await.redeem(&address.0, &request.data.id, request.auth.device.id).await?.into())
}
