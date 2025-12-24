use async_trait::async_trait;
use gem_rewards::{RedemptionAsset, RedemptionRequest, RedemptionService};
use primitives::rewards::RedemptionStatus;
use std::error::Error;
use std::sync::Arc;
use storage::{Database, RedemptionUpdate};
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
        let mut client = self.database.client()?;
        let redemption = client.rewards().get_redemption(payload.redemption_id)?;

        if redemption.status == RedemptionStatus::Completed.as_ref() {
            return Ok(true);
        }

        let recipient_address = client.rewards().get_address_by_username(&redemption.username)?;
        let option = client.rewards().get_redemption_option(&redemption.option_id)?;

        let asset = option.asset.map(|asset| RedemptionAsset {
            asset,
            value: option.value.clone(),
        });

        let request = RedemptionRequest { recipient_address, asset };

        match self.redemption_service.process_redemption(request).await {
            Ok(result) => {
                let updates = vec![
                    RedemptionUpdate::TransactionId(result.transaction_id),
                    RedemptionUpdate::Status(RedemptionStatus::Completed.as_ref().to_string()),
                ];

                client.rewards().update_redemption(payload.redemption_id, updates)?;
                Ok(true)
            }
            Err(e) => {
                let updates = vec![
                    RedemptionUpdate::Status(RedemptionStatus::Failed.as_ref().to_string()),
                    RedemptionUpdate::Error(e.to_string()),
                ];

                client.rewards().update_redemption(payload.redemption_id, updates)?;
                Ok(false)
            }
        }
    }
}
