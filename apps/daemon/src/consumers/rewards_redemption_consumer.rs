use async_trait::async_trait;
use gem_rewards::{RedemptionAsset, RedemptionRequest, RedemptionService};
use primitives::rewards::RedemptionStatus as PrimitiveRedemptionStatus;
use std::error::Error;
use std::sync::Arc;
use storage::sql_types::RedemptionStatus;
use storage::{Database, RedemptionUpdate, RewardsRedemptionsRepository, RewardsRepository};
use streamer::RewardsRedemptionPayload;
use streamer::consumer::MessageConsumer;

pub struct RewardsRedemptionConsumer<S: RedemptionService> {
    database: Database,
    redemption_service: Arc<S>,
}

impl<S: RedemptionService> RewardsRedemptionConsumer<S> {
    pub fn new(database: Database, redemption_service: Arc<S>) -> Self {
        Self { database, redemption_service }
    }
}

#[async_trait]
impl<S: RedemptionService> MessageConsumer<RewardsRedemptionPayload, bool> for RewardsRedemptionConsumer<S> {
    async fn should_process(&self, _payload: RewardsRedemptionPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: RewardsRedemptionPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let redemption = self.database.rewards_redemptions()?.get_redemption(payload.redemption_id)?;

        if *redemption.status == PrimitiveRedemptionStatus::Completed {
            return Ok(true);
        }

        let recipient_address = self.database.rewards()?.get_address_by_username(&redemption.username)?;
        let option = self.database.rewards_redemptions()?.get_redemption_option(&redemption.option_id)?;

        let asset = option.asset.map(|asset| RedemptionAsset {
            asset,
            value: option.value.clone(),
        });

        let request = RedemptionRequest { recipient_address, asset };

        match self.redemption_service.process_redemption(request).await {
            Ok(result) => {
                let updates = vec![
                    RedemptionUpdate::TransactionId(result.transaction_id),
                    RedemptionUpdate::Status(RedemptionStatus::Completed),
                ];

                self.database.rewards_redemptions()?.update_redemption(payload.redemption_id, updates)?;
                Ok(true)
            }
            Err(e) => {
                let updates = vec![
                    RedemptionUpdate::Status(RedemptionStatus::Failed),
                    RedemptionUpdate::Error(e.to_string()),
                ];

                self.database.rewards_redemptions()?.update_redemption(payload.redemption_id, updates)?;
                Ok(false)
            }
        }
    }
}
