use api_connector::PusherClient;
use price_alert::{client::PriceAlertRules, PriceAlertClient};
use settings::AlerterRules;

pub struct PriceAlertSender {
    price_alert_client: PriceAlertClient,
    pusher_client: PusherClient,
    topic: String,
    rules: AlerterRules,
}

impl PriceAlertSender {
    pub fn new(price_alert_client: PriceAlertClient, pusher_client: PusherClient, rules: AlerterRules, topic: String) -> Self {
        Self {
            price_alert_client,
            pusher_client,
            topic,
            rules,
        }
    }

    pub async fn run(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let rules = PriceAlertRules {
            price_change_increase: self.rules.price_increase_percent,
            price_change_decrease: self.rules.price_decrease_percent,
        };

        let price_alert_notifications = self.price_alert_client.get_devices_to_alert(rules).await?;

        let notifications = self
            .price_alert_client
            .get_notifications_for_price_alerts(price_alert_notifications, self.topic.clone());

        if notifications.is_empty() {
            return Ok(0);
        }

        match self.pusher_client.push_notifications(notifications.clone()).await {
            Ok(_) => {}
            Err(e) => {
                println!("alerter failed to send notification: {:?}", e);
            }
        }

        Ok(notifications.len())
    }
}
