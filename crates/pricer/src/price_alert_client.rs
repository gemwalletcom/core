use chrono::{Duration, Utc};
use gem_tracing::info_with_fields;
use localizer::{LanguageLocalizer, LanguageNotification};
use number_formatter::NumberFormatter;
use primitives::{
    Asset, DEFAULT_FIAT_CURRENCY, Device, GorushNotification, Price, PriceAlert, PriceAlertDirection, PriceAlertType, PriceAlerts, PriceData, PushNotification,
    PushNotificationAsset, PushNotificationTypes,
};
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration as StdDuration;
use storage::{AssetsRepository, Database, PriceAlertsRepository};

const DEFAULT_RANK: i32 = 1000;

#[derive(Clone)]
pub struct PriceAlertClient {
    database: Database,
}

#[derive(Clone, Debug)]
pub struct PriceAlertNotification {
    pub device: Device,
    pub asset: Asset,
    pub price: Price,
    pub alert_type: PriceAlertType,
    pub price_alert: PriceAlert,
    pub milestone: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct PriceAlertRules {
    pub notification_cooldown: StdDuration,
    pub price_change_threshold: f64,
    pub rank_divisor: f64,
    pub milestones: Vec<f64>,
}

struct AlertResult {
    alert_type: PriceAlertType,
    milestone: Option<f64>,
}

impl PriceAlertRules {
    fn calculate_threshold(&self, rank: i32) -> f64 {
        let rank = if rank > 0 { rank } else { DEFAULT_RANK };
        self.price_change_threshold * (1.0 + (rank as f64).ln() / self.rank_divisor)
    }

    fn find_crossed_milestone(&self, price_24h_ago: f64, current_price: f64) -> Option<f64> {
        if current_price <= price_24h_ago {
            return None;
        }

        self.milestones
            .iter()
            .find(|&&milestone| price_24h_ago < milestone && current_price >= milestone)
            .copied()
    }
}

fn calculate_price_24h_ago(current_price: f64, change_percent: f64) -> f64 {
    let divisor = 1.0 + change_percent / 100.0;
    if divisor <= 0.0 {
        return current_price;
    }
    current_price / divisor
}

impl PriceAlertClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_price_alerts(&self, device_id: &str, asset_id: Option<&str>) -> Result<PriceAlerts, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .price_alerts()?
            .get_price_alerts_for_device_id(device_id, asset_id)?
            .into_iter()
            .map(|x| x.price_alert)
            .collect())
    }

    pub async fn add_price_alerts(&self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.price_alerts()?.add_price_alerts(device_id, price_alerts)?)
    }

    pub async fn delete_price_alerts(&self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ids = price_alerts.iter().map(|x| x.id()).collect::<HashSet<_>>().into_iter().collect();
        Ok(self.database.price_alerts()?.delete_price_alerts(device_id, ids)?)
    }

    pub async fn get_devices_to_alert(&self, rules: PriceAlertRules) -> Result<Vec<PriceAlertNotification>, Box<dyn Error + Send + Sync>> {
        let now = Utc::now();
        let cooldown = Duration::seconds(rules.notification_cooldown.as_secs() as i64);
        let after_notified_at = now - cooldown;
        let price_alerts = self.database.price_alerts()?.get_price_alerts(after_notified_at.naive_utc())?;

        let mut results: Vec<PriceAlertNotification> = Vec::new();
        let mut price_alert_ids: HashSet<String> = HashSet::new();

        for (price_alert, price_data, device) in price_alerts {
            if let Some(alert_result) = self.get_price_alert_type(&price_alert, &price_data, &rules) {
                let notification = self.price_alert_notification(device, &price_data, price_alert.clone(), alert_result.alert_type, alert_result.milestone)?;
                price_alert_ids.insert(price_alert.id());
                results.push(notification);
            }
        }

        self.database
            .price_alerts()?
            .update_price_alerts_set_notified_at(price_alert_ids.into_iter().collect(), now.naive_utc())?;
        Ok(results)
    }

    fn get_price_alert_type(&self, price_alert: &PriceAlert, price_data: &PriceData, rules: &PriceAlertRules) -> Option<AlertResult> {
        // User-defined price target
        if let Some(target_price) = price_alert.price {
            let direction = price_alert.price_direction.clone()?;
            let alert_type = match direction {
                PriceAlertDirection::Up if price_data.price >= target_price => Some(PriceAlertType::PriceUp),
                PriceAlertDirection::Down if price_data.price <= target_price => Some(PriceAlertType::PriceDown),
                _ => None,
            };
            return alert_type.map(|t| AlertResult {
                alert_type: t,
                milestone: None,
            });
        }

        // User-defined percent change
        if let Some(target_percent) = price_alert.price_percent_change {
            let direction = price_alert.price_direction.clone()?;
            let alert_type = match direction {
                PriceAlertDirection::Up if price_data.price_change_percentage_24h >= target_percent => Some(PriceAlertType::PricePercentChangeUp),
                PriceAlertDirection::Down if price_data.price_change_percentage_24h <= -target_percent => Some(PriceAlertType::PricePercentChangeDown),
                _ => None,
            };
            return alert_type.map(|t| AlertResult {
                alert_type: t,
                milestone: None,
            });
        }

        // All-time high check
        if price_data.all_time_high > 0.0 && price_data.price > price_data.all_time_high {
            return Some(AlertResult {
                alert_type: PriceAlertType::AllTimeHigh,
                milestone: None,
            });
        }

        // Price milestone check
        let price_24h_ago = calculate_price_24h_ago(price_data.price, price_data.price_change_percentage_24h);
        if let Some(milestone) = rules.find_crossed_milestone(price_24h_ago, price_data.price) {
            return Some(AlertResult {
                alert_type: PriceAlertType::PriceMilestone,
                milestone: Some(milestone),
            });
        }

        // Dynamic threshold based on rank
        let threshold = rules.calculate_threshold(price_data.market_cap_rank);
        if price_data.price_change_percentage_24h > threshold {
            return Some(AlertResult {
                alert_type: PriceAlertType::PriceChangesUp,
                milestone: None,
            });
        }
        if price_data.price_change_percentage_24h < -threshold {
            return Some(AlertResult {
                alert_type: PriceAlertType::PriceChangesDown,
                milestone: None,
            });
        }

        None
    }

    fn price_alert_notification(
        &self,
        device: Device,
        price_data: &PriceData,
        price_alert: PriceAlert,
        alert_type: PriceAlertType,
        milestone: Option<f64>,
    ) -> Result<PriceAlertNotification, Box<dyn Error + Send + Sync>> {
        let asset = self.database.assets()?.get_asset(&price_alert.asset_id.to_string())?;
        let base_rate = self.database.fiat()?.get_fiat_rate(DEFAULT_FIAT_CURRENCY)?;
        let rate = self.database.fiat()?.get_fiat_rate(&device.currency)?;

        let price = Price::new(price_data.price, price_data.price_change_percentage_24h, price_data.last_updated_at);
        let price = price.new_with_rate(base_rate.rate, rate.rate);

        Ok(PriceAlertNotification {
            device,
            asset,
            price,
            alert_type,
            price_alert,
            milestone,
        })
    }

    pub fn get_notifications_for_price_alerts(&self, notifications: Vec<PriceAlertNotification>) -> Vec<GorushNotification> {
        let mut results = vec![];
        let formatter = NumberFormatter::new();

        for alert in notifications {
            if !alert.device.can_receive_price_alerts() {
                continue;
            }

            let price = match formatter.currency(alert.price.price, &alert.device.currency) {
                Some(p) => p,
                None => {
                    info_with_fields!("unknown_currency_symbol", currency = &alert.device.currency);
                    continue;
                }
            };

            let change = formatter.percent(alert.price.price_change_percentage_24h, alert.device.locale.as_str());
            let localizer = LanguageLocalizer::new_with_language(&alert.device.locale);

            let message: LanguageNotification = match alert.alert_type {
                PriceAlertType::PriceUp | PriceAlertType::PriceDown => localizer.price_alert_target(&alert.asset.full_name(), &price, &change),
                PriceAlertType::PriceChangesUp | PriceAlertType::PricePercentChangeUp => localizer.price_alert_up(&alert.asset.full_name(), &price, &change),
                PriceAlertType::PriceChangesDown | PriceAlertType::PricePercentChangeDown => {
                    localizer.price_alert_down(&alert.asset.full_name(), &price, &change)
                }
                PriceAlertType::AllTimeHigh => localizer.price_alert_all_time_high(&alert.asset.name, &price),
                PriceAlertType::PriceMilestone => {
                    let milestone_price = alert
                        .milestone
                        .and_then(|m| formatter.currency(m, &alert.device.currency))
                        .unwrap_or_else(|| price.clone());
                    localizer.price_alert_milestone(&alert.asset.full_name(), &milestone_price, &change)
                }
            };

            let data = PushNotification {
                data: serde_json::to_value(&PushNotificationAsset {
                    asset_id: alert.asset.id.to_string(),
                })
                .ok(),
                notification_type: PushNotificationTypes::PriceAlert,
            };

            results.push(GorushNotification::from_device(alert.device.clone(), message.title, message.description, data));
        }

        results
    }
}
