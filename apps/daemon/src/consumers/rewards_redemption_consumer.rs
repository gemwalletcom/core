use async_trait::async_trait;
use gem_rewards::{RedemptionAsset, RedemptionRequest, RedemptionService};
use gem_tracing::info_with_fields;
use primitives::rewards::RedemptionStatus as PrimitiveRedemptionStatus;
use primitives::{NotificationRewardsRedeemMetadata, NotificationType};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use storage::sql_types::RedemptionStatus;
use storage::{Database, RedemptionUpdate, RewardsRedemptionsRepository, RewardsRepository};
use streamer::consumer::MessageConsumer;
use streamer::{InAppNotificationPayload, RewardsRedemptionPayload, StreamProducer, StreamProducerQueue};

pub struct RedemptionRetryConfig {
    pub max_retries: u32,
    pub delay: Duration,
    pub errors: Vec<String>,
}

pub struct RewardsRedemptionConsumer<S: RedemptionService> {
    database: Database,
    redemption_service: Arc<S>,
    retry_config: RedemptionRetryConfig,
    stream_producer: StreamProducer,
}

impl<S: RedemptionService> RewardsRedemptionConsumer<S> {
    pub fn new(database: Database, redemption_service: Arc<S>, retry_config: RedemptionRetryConfig, stream_producer: StreamProducer) -> Self {
        Self {
            database,
            redemption_service,
            retry_config,
            stream_producer,
        }
    }

    async fn process_with_retry(&self, request: RedemptionRequest) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut attempt = 0;
        loop {
            match self.redemption_service.process_redemption(request.clone()).await {
                Ok(result) => return Ok(result.transaction_id),
                Err(e) => {
                    let is_retryable = self.retry_config.errors.iter().any(|p| e.to_string().contains(p));
                    if attempt < self.retry_config.max_retries && is_retryable {
                        attempt += 1;
                        tokio::time::sleep(self.retry_config.delay).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }
}

#[async_trait]
impl<S: RedemptionService> MessageConsumer<RewardsRedemptionPayload, PrimitiveRedemptionStatus> for RewardsRedemptionConsumer<S> {
    async fn should_process(&self, payload: RewardsRedemptionPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let redemption = self.database.rewards_redemptions()?.get_redemption(payload.redemption_id)?;
        Ok(*redemption.status == PrimitiveRedemptionStatus::Pending)
    }

    async fn process(&self, payload: RewardsRedemptionPayload) -> Result<PrimitiveRedemptionStatus, Box<dyn Error + Send + Sync>> {
        let redemption = self.database.rewards_redemptions()?.get_redemption(payload.redemption_id)?;

        self.database
            .rewards_redemptions()?
            .update_redemption(payload.redemption_id, vec![RedemptionUpdate::Status(RedemptionStatus::Processing)])?;

        let recipient_address = self.database.rewards()?.get_address_by_username(&redemption.username)?;
        let option = self.database.rewards_redemptions()?.get_redemption_option(&redemption.option_id)?;

        let asset_id = option.asset.as_ref().map(|a| a.id.clone());
        let asset_id_str = asset_id.as_ref().map(|a| a.to_string());
        let value = option.value.clone();
        let points = option.points;

        let asset = option.asset.map(|asset| RedemptionAsset {
            asset,
            value: option.value.clone(),
        });

        let request = RedemptionRequest { recipient_address, asset };

        match self.process_with_retry(request).await {
            Ok(transaction_id) => {
                let updates = vec![
                    RedemptionUpdate::TransactionId(transaction_id.clone()),
                    RedemptionUpdate::Status(RedemptionStatus::Completed),
                ];
                self.database.rewards_redemptions()?.update_redemption(payload.redemption_id, updates)?;

                if let Some(id) = &asset_id {
                    let metadata = NotificationRewardsRedeemMetadata {
                        transaction_id: transaction_id.clone(),
                        points,
                        value: value.clone(),
                    };
                    let notification = InAppNotificationPayload::new_with_asset(redemption.wallet_id, id.to_string(), NotificationType::RewardsRedeemed, serde_json::to_value(metadata).ok());
                    self.stream_producer.publish_in_app_notifications(vec![notification]).await?;
                }

                info_with_fields!(
                    "redemption completed",
                    id = payload.redemption_id,
                    asset = asset_id_str.as_deref().unwrap_or("none"),
                    value = value,
                    tx_id = transaction_id
                );
                Ok(PrimitiveRedemptionStatus::Completed)
            }
            Err(e) => {
                let error_msg = e.to_string();
                let updates = vec![RedemptionUpdate::Status(RedemptionStatus::Failed), RedemptionUpdate::Error(error_msg.clone())];
                self.database.rewards_redemptions()?.update_redemption(payload.redemption_id, updates)?;
                info_with_fields!(
                    "redemption failed",
                    id = payload.redemption_id,
                    asset = asset_id_str.as_deref().unwrap_or("none"),
                    value = value,
                    error = error_msg
                );
                Ok(PrimitiveRedemptionStatus::Failed)
            }
        }
    }
}
