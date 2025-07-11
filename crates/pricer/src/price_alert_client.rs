use chrono::{Duration, Utc};
use localizer::{LanguageLocalizer, LanguageNotification};
use number_formatter::NumberFormatter;
use primitives::{
    Asset, Device, GorushNotification, Price, PriceAlert, PriceAlertDirection, PriceAlertType, PriceAlerts, PushNotification, PushNotificationAsset,
    PushNotificationTypes, DEFAULT_FIAT_CURRENCY,
};
use std::collections::HashSet;
use std::error::Error;
use storage::{DatabaseClient, DatabaseClientExt};

#[allow(dead_code)]
pub struct PriceAlertClient {
    database: DatabaseClient,
}

#[derive(Clone, Debug)]
pub struct PriceAlertNotification {
    pub device: Device,
    pub asset: Asset,
    pub price: Price,
    pub alert_type: PriceAlertType,
    pub price_alert: PriceAlert,
}

#[derive(Clone, Debug)]
pub struct PriceAlertRules {
    pub price_change_increase: f64,
    pub price_change_decrease: f64,
}

impl PriceAlertClient {
    pub async fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub async fn get_price_alerts(&mut self, device_id: &str) -> Result<PriceAlerts, Box<dyn Error + Send + Sync>> {
        let device_alerts = self.database.repositories().price_alerts().get_price_alerts_for_device_id(device_id)?;
        Ok(device_alerts.into_iter().map(|x| x.price_alert).collect())
    }

    pub async fn add_price_alerts(&mut self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.repositories().price_alerts().add_price_alerts(device_id, price_alerts)?)
    }

    pub async fn delete_price_alerts(&mut self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ids = price_alerts.iter().map(|x| x.id()).collect::<HashSet<_>>().into_iter().collect();
        Ok(self.database.repositories().price_alerts().delete_price_alerts(device_id, ids)?)
    }

    pub async fn get_devices_to_alert(&mut self, rules: PriceAlertRules) -> Result<Vec<PriceAlertNotification>, Box<dyn Error + Send + Sync>> {
        let now = Utc::now();
        let after_notified_at = now - Duration::days(1);
        let price_alerts = self.database.repositories().price_alerts().get_price_alerts(after_notified_at.naive_utc())?;

        let mut results: Vec<PriceAlertNotification> = Vec::new();
        let mut price_alert_ids: HashSet<String> = HashSet::new();

        for (price_alert, price, device) in price_alerts {
            if let Some(alert) = self.get_price_alert_type(&price_alert, price.clone(), rules.clone()) {
                let notification = self.price_alert_notification(device, price, price_alert.clone(), alert)?;
                price_alert_ids.insert(price_alert.id());
                results.push(notification);
            }
        }
        self.database
            .repositories()
            .price_alerts()
            .update_price_alerts_set_notified_at(price_alert_ids.into_iter().collect(), now.naive_utc())?;
        Ok(results)
    }

    fn get_price_alert_type(&self, price_alert: &primitives::PriceAlert, price: primitives::Price, rules: PriceAlertRules) -> Option<PriceAlertType> {
        if let Some(price_alert_price) = price_alert.price {
            let direction = price_alert.price_direction.clone()?;
            let current_price = price.price;
            return match direction {
                PriceAlertDirection::Up if current_price >= price_alert_price => Some(PriceAlertType::PriceUp),
                PriceAlertDirection::Down if current_price <= price_alert_price => Some(PriceAlertType::PriceDown),
                _ => None,
            };
        } else if let Some(price_alert_percent) = price_alert.price_percent_change {
            let direction = price_alert.price_direction.clone()?;
            let price_change_percentage_24h = price.price_change_percentage_24h;
            return match direction {
                PriceAlertDirection::Up if price_change_percentage_24h >= price_alert_percent => Some(PriceAlertType::PricePercentChangeUp),
                PriceAlertDirection::Down if price_change_percentage_24h <= -price_alert_percent => Some(PriceAlertType::PricePercentChangeDown),
                _ => None,
            };
        } else if price.price_change_percentage_24h > rules.price_change_increase {
            return Some(PriceAlertType::PriceChangesUp);
        } else if price.price_change_percentage_24h < -rules.price_change_decrease {
            return Some(PriceAlertType::PriceChangesDown);
        }
        None
    }

    fn price_alert_notification(
        &mut self,
        device: primitives::Device,
        price: primitives::Price,
        price_alert: PriceAlert,
        alert_type: PriceAlertType,
    ) -> Result<PriceAlertNotification, Box<dyn Error + Send + Sync>> {
        let asset = self.database.repositories().assets().get_asset(&price_alert.asset_id.to_string())?;
        let base_rate = self.database.repositories().fiat().get_fiat_rate(DEFAULT_FIAT_CURRENCY)?;
        let rate = self.database.repositories().fiat().get_fiat_rate(&device.currency)?;
        let price = price.new_with_rate(base_rate.rate, rate.rate);

        let notification = PriceAlertNotification {
            device,
            asset,
            price,
            alert_type,
            price_alert,
        };
        Ok(notification)
    }

    pub fn get_notifications_for_price_alerts(&mut self, notifications: Vec<PriceAlertNotification>) -> Vec<GorushNotification> {
        let mut results = vec![];

        let formatter = NumberFormatter::new();

        for price_alert in notifications {
            let price = formatter.currency(price_alert.price.price, &price_alert.device.currency);
            if price.is_none() {
                println!("Unknown currency symbol: {}", &price_alert.device.currency);
                continue;
            }
            let price_change = formatter.percent(price_alert.price.price_change_percentage_24h, price_alert.device.locale.as_str());

            let language_localizer = LanguageLocalizer::new_with_language(&price_alert.device.locale);
            let notification_message: LanguageNotification = match price_alert.alert_type {
                PriceAlertType::PriceChangesUp | PriceAlertType::PriceUp | PriceAlertType::PricePercentChangeUp => {
                    language_localizer.price_alert_up(&price_alert.asset.full_name(), price.unwrap().as_str(), price_change.as_str())
                }
                PriceAlertType::PriceChangesDown | PriceAlertType::PriceDown | PriceAlertType::PricePercentChangeDown => {
                    language_localizer.price_alert_down(&price_alert.asset.full_name(), price.unwrap().as_str(), price_change.as_str())
                }
                PriceAlertType::AllTimeHigh => language_localizer.price_alert_all_time_high(&price_alert.asset.name, price.unwrap().as_str()),
            };
            let price_alert_data = PushNotificationAsset {
                asset_id: price_alert.asset.id.to_string(),
            };
            let data = PushNotification {
                data: serde_json::to_value(&price_alert_data).ok(),
                notification_type: PushNotificationTypes::PriceAlert,
            };
            let notification = GorushNotification::new(
                vec![price_alert.device.token.clone()],
                price_alert.device.platform,
                notification_message.title,
                notification_message.description,
                data,
            );
            results.push(notification);
        }
        results
    }
}
