use api_connector::pusher::model::Notification;
use api_connector::PusherClient;
use pricer::price_alert_client::PriceAlertRules;
use pricer::PriceAlertClient;
use settings::AlerterRules;

pub struct PriceAlertSender {
    price_alert_client: PriceAlertClient,
    pusher_client: PusherClient,
    rules: AlerterRules,
}

impl PriceAlertSender {
    pub fn new(price_alert_client: PriceAlertClient, pusher_client: PusherClient, rules: AlerterRules) -> Self {
        Self {
            price_alert_client,
            pusher_client,
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

        self.notify(notifications).await
    }

    pub async fn notify(&mut self, notifications: Vec<Notification>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
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
