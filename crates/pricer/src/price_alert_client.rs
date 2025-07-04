use chrono::{Duration, NaiveDateTime, Utc};
use localizer::{LanguageLocalizer, LanguageNotification};
use number_formatter::NumberFormatter;
use primitives::{
    Asset, Device, GorushNotification, Price, PriceAlert, PriceAlertDirection, PriceAlertType, PriceAlerts, PushNotification, PushNotificationAsset,
    PushNotificationTypes, DEFAULT_FIAT_CURRENCY,
};
use std::collections::HashSet;
use std::error::Error;
use storage::DatabaseClient;

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

    pub async fn get_price_alerts(&mut self, device_id: &str) -> Result<PriceAlerts, Box<dyn Error>> {
        let device = self.database.get_device(device_id)?;
        let values = self
            .database
            .get_price_alerts_for_device_id(device.id)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect::<_>();
        Ok(values)
    }

    pub async fn add_price_alerts(&mut self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, Box<dyn Error>> {
        let device = self.database.get_device(device_id)?;
        let values = price_alerts
            .into_iter()
            .map(|x| storage::models::PriceAlert::new_price_alert(x, device.id))
            .collect::<_>();
        Ok(self.database.add_price_alerts(values)?)
    }

    pub async fn delete_price_alerts(&mut self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, Box<dyn Error>> {
        let device = self.database.get_device(device_id)?;
        let ids = price_alerts.iter().map(|x| x.id()).collect::<HashSet<_>>().into_iter().collect();
        Ok(self.database.delete_price_alerts(device.id, ids)?)
    }

    pub async fn get_devices_to_alert(&mut self, rules: PriceAlertRules) -> Result<Vec<PriceAlertNotification>, Box<dyn Error + Send + Sync>> {
        let now = Utc::now();
        let after_notified_at = now - Duration::days(1);
        let price_alerts = self.database.get_price_alerts_with_prices(after_notified_at.naive_utc())?;

        let mut results: Vec<PriceAlertNotification> = Vec::new();
        let mut price_alert_ids: HashSet<String> = HashSet::new();

        for (price_alert, price) in price_alerts {
            let device_id = price_alert.device_id;
            let price_alert = price_alert.as_primitive();

            if let Some(alert) = self.get_price_alert_type(&price_alert, price.clone(), rules.clone()) {
                let notification = self.price_alert_notification(device_id, price.as_price_primitive(), price_alert.clone(), alert)?;
                price_alert_ids.insert(price_alert.id());
                results.push(notification);
            }
        }
        self.database
            .update_price_alerts_set_notified_at(price_alert_ids.into_iter().collect(), now.naive_utc())?;
        Ok(results)
    }

    fn get_price_alert_type(&self, price_alert: &primitives::PriceAlert, price: storage::models::Price, rules: PriceAlertRules) -> Option<PriceAlertType> {
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
            let price_change_percentage_24h = price.clone().price_change_percentage_24h;
            return match direction {
                PriceAlertDirection::Up if price_change_percentage_24h >= price_alert_percent => Some(PriceAlertType::PricePercentChangeUp),
                PriceAlertDirection::Down if price_change_percentage_24h <= -price_alert_percent => Some(PriceAlertType::PricePercentChangeDown),
                _ => None,
            };
        } else if Self::is_within_past(price.clone().all_time_high_date, Duration::hours(12)) {
            return Some(PriceAlertType::AllTimeHigh);
        } else if price.clone().price_change_percentage_24h > rules.price_change_increase {
            return Some(PriceAlertType::PriceChangesUp);
        } else if price.clone().price_change_percentage_24h < -rules.price_change_decrease {
            return Some(PriceAlertType::PriceChangesDown);
        }
        None
    }

    fn is_within_past(date_time: Option<NaiveDateTime>, duration: Duration) -> bool {
        if let Some(date_time) = date_time {
            return date_time >= (Utc::now().naive_utc() - duration) && date_time <= Utc::now().naive_utc();
        }
        false
    }

    fn price_alert_notification(
        &mut self,
        device_id: i32,
        price: Price,
        price_alert: PriceAlert,
        alert_type: PriceAlertType,
    ) -> Result<PriceAlertNotification, Box<dyn Error + Send + Sync>> {
        let asset = self.database.get_asset(&price_alert.asset_id.to_string())?.as_primitive();
        let device = self.database.get_device_by_id(device_id)?.as_primitive();
        let base_rate = self.database.get_fiat_rate(DEFAULT_FIAT_CURRENCY)?;
        let rate = self.database.get_fiat_rate(&device.currency)?;
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
