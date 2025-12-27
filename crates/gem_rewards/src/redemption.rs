use primitives::rewards::{RedemptionResponse, RedemptionResult};
use storage::{DatabaseClient, DatabaseError, RewardsRedemptionsRepository};

pub fn redeem_points(database: &mut DatabaseClient, username: &str, option_id: &str, device_id: i32) -> Result<RedemptionResponse, DatabaseError> {
    let redemption = RewardsRedemptionsRepository::add_redemption(database, username, option_id, device_id)?;

    Ok(RedemptionResponse {
        result: RedemptionResult {
            redemption: redemption.clone(),
        },
        redemption_id: redemption.id,
    })
}
