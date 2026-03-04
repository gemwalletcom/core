use crate::devices::RewardsClient;
use crate::responders::{ApiError, ApiResponse};
use primitives::ReferralLeaderboard;
use rocket::{State, get};
use tokio::sync::Mutex;

#[get("/rewards/leaderboard")]
pub async fn get_rewards_leaderboard(client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<ReferralLeaderboard>, ApiError> {
    Ok(client.lock().await.get_rewards_leaderboard()?.into())
}
