mod client;
mod redemption_client;

pub use client::RewardsClient;
pub use redemption_client::RewardsRedemptionClient;

use crate::auth::Authenticated;
use crate::params::AddressParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::rewards::{RedemptionRequest, RedemptionResult, RewardRedemptionOption};
use primitives::{ReferralCode, ReferralLeaderboard, RewardEvent, Rewards};
use rocket::{State, get, post};
use tokio::sync::Mutex;

#[get("/rewards/leaderboard")]
pub async fn get_rewards_leaderboard(client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<ReferralLeaderboard>, ApiError> {
    Ok(client.lock().await.get_rewards_leaderboard()?.into())
}

#[get("/rewards/redemptions/<code>", rank = 1)]
pub async fn get_rewards_redemption_option(code: String, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<RewardRedemptionOption>, ApiError> {
    Ok(client.lock().await.get_rewards_redemption_option(&code)?.into())
}

#[get("/rewards/<address>/events")]
pub async fn get_rewards_events(address: AddressParam, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<Vec<RewardEvent>>, ApiError> {
    Ok(client.lock().await.get_rewards_events(&address.0)?.into())
}

#[get("/rewards/<address>")]
pub async fn get_rewards(address: AddressParam, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<Rewards>, ApiError> {
    Ok(client.lock().await.get_rewards(&address.0)?.into())
}

#[post("/rewards/referrals/create", format = "json", data = "<request>")]
pub async fn create_referral(
    request: Authenticated<ReferralCode>,
    ip: std::net::IpAddr,
    client: &State<Mutex<RewardsClient>>,
) -> Result<ApiResponse<Rewards>, ApiError> {
    Ok(client
        .lock()
        .await
        .create_username(
            &request.auth.address,
            &request.data.code,
            request.auth.device.id,
            &ip.to_string(),
            &request.auth.device.locale,
        )
        .await?
        .into())
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
    if !address.0.eq_ignore_ascii_case(&request.auth.address) {
        return Err(ApiError::BadRequest("Address mismatch".to_string()));
    }
    Ok(client
        .lock()
        .await
        .redeem(&request.auth.address, &request.data.id, request.auth.device.id)
        .await?
        .into())
}
