use pricer::PriceAlertClient;
use pricer::price_alert_client::PriceAlertRules;
use settings::AlerterRules;
use streamer::{NotificationsPayload, StreamProducer, StreamProducerQueue};

pub struct PriceAlertSender {
    price_alert_client: PriceAlertClient,
    stream_producer: StreamProducer,
    rules: AlerterRules,
}

impl PriceAlertSender {
    pub fn new(price_alert_client: PriceAlertClient, stream_producer: StreamProducer, rules: AlerterRules) -> Self {
        Self {
            price_alert_client,
            stream_producer,
            rules,
        }
    }

    pub async fn run_observer(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let rules = PriceAlertRules {
            price_change_increase: self.rules.price_increase_percent,
            price_change_decrease: self.rules.price_decrease_percent,
        };
        let price_alert_notifications = self.price_alert_client.get_devices_to_alert(rules).await?;
        let notifications = self.price_alert_client.get_notifications_for_price_alerts(price_alert_notifications);
        self.stream_producer
            .publish_notifications_price_alerts(NotificationsPayload::new(notifications.clone()))
            .await?;
        Ok(notifications.len())
    }
}
