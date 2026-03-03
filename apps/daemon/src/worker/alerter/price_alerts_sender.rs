use pricer::PriceAlertClient;
use pricer::price_alert_client::PriceAlertRules;
use primitives::ConfigKey;
use storage::{ConfigCacher, Database};
use streamer::{NotificationsPayload, StreamProducer, StreamProducerQueue};

pub struct PriceAlertSender {
    config: ConfigCacher,
    price_alert_client: PriceAlertClient,
    stream_producer: StreamProducer,
}

impl PriceAlertSender {
    pub fn new(database: Database, price_alert_client: PriceAlertClient, stream_producer: StreamProducer) -> Self {
        let config = ConfigCacher::new(database);
        Self {
            config,
            price_alert_client,
            stream_producer,
        }
    }

    pub async fn run_observer(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let notification_cooldown = self.config.get_duration(ConfigKey::AlerterPriceAlertsCooldown)?;
        let price_change_threshold = self.config.get_f64(ConfigKey::AlerterPriceAlertsThreshold)?;
        let rank_divisor = self.config.get_f64(ConfigKey::AlerterPriceAlertsRankDivisor)?;
        let milestones = self.config.get_vec::<f64>(ConfigKey::AlerterPriceAlertsMilestones)?;

        let rules = PriceAlertRules {
            notification_cooldown,
            price_change_threshold,
            rank_divisor,
            milestones,
        };

        let price_alert_notifications = self.price_alert_client.get_devices_to_alert(rules).await?;
        let notifications = self.price_alert_client.get_notifications_for_price_alerts(price_alert_notifications);
        self.stream_producer
            .publish_notifications_price_alerts(NotificationsPayload::new(notifications.clone()))
            .await?;
        Ok(notifications.len())
    }
}
