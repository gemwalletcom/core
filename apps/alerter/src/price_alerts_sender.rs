use api_connector::{pusher::model::Notification, PusherClient};
use localizer::LanguageLocalizer;
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

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let rules = PriceAlertRules {
            price_change_increase: self.rules.price_increase_percent,
        };

        let price_alert_notifications = self.price_alert_client.get_devices_to_alert(rules).await?;

        println!("alerter found devices to alert: {:?}", price_alert_notifications.len());

        for price_alert_notification in price_alert_notifications {
            let mut notifications = vec![];

            let message = LanguageLocalizer::new_with_language(&price_alert_notification.device.locale).price_alert_up(
                &price_alert_notification.asset.full_name(),
                price_alert_notification.price.price.to_string().as_str(),
                price_alert_notification.price.price_change_percentage_24h.to_string().as_str(),
            );

            let notification = Notification {
                tokens: vec![price_alert_notification.device.token.clone()],
                platform: price_alert_notification.device.platform.as_i32(),
                title: message.title,
                message: message.description,
                topic: Some(self.topic.clone()),
                data: None,
            };
            notifications.push(notification);

            match self.pusher_client.push_notifications(notifications).await {
                Ok(_) => {}
                Err(e) => {
                    println!("alerter failed to send notification: {:?}", e);
                }
            }
        }

        Ok(())
    }
}
