use crate::{DatabaseError, DieselResultExt};
use chrono::NaiveDateTime;
use primitives::{Device, DevicePriceAlert, PriceAlert, PriceAlerts, PriceData};
use std::collections::HashMap;

use crate::DatabaseClient;
use crate::database::devices::DevicesStore;
use crate::database::price_alerts::PriceAlertsStore;
use crate::repositories::prices_repository::PricesRepository;

pub trait PriceAlertsRepository {
    fn get_price_alerts(&mut self, after_notified_at: NaiveDateTime) -> Result<Vec<(PriceAlert, PriceData, Device)>, DatabaseError>;
    fn get_price_alerts_for_device_id(&mut self, device_id: &str, asset_id: Option<&str>) -> Result<Vec<DevicePriceAlert>, DatabaseError>;
    fn add_price_alerts(&mut self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, DatabaseError>;
    fn delete_price_alerts(&mut self, device_id: &str, ids: Vec<String>) -> Result<usize, DatabaseError>;
    fn update_price_alerts_set_notified_at(&mut self, ids: Vec<String>, last_notified_at: NaiveDateTime) -> Result<usize, DatabaseError>;
}

impl PriceAlertsRepository for DatabaseClient {
    fn get_price_alerts(&mut self, after_notified_at: NaiveDateTime) -> Result<Vec<(PriceAlert, PriceData, Device)>, DatabaseError> {
        let alerts = PriceAlertsStore::get_price_alerts(self, after_notified_at)?;
        if alerts.is_empty() {
            return Ok(vec![]);
        }
        let asset_ids: Vec<String> = alerts
            .iter()
            .map(|(a, _)| a.asset_id.to_string())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let primary: HashMap<String, PriceData> = self
            .get_primary_prices(&asset_ids)?
            .into_iter()
            .map(|(id, row)| (id.to_string(), row.as_price_data()))
            .collect();
        Ok(alerts
            .into_iter()
            .filter_map(|(alert, device)| {
                primary
                    .get(&alert.asset_id.to_string())
                    .map(|price| (alert.as_primitive(), price.clone(), device.as_primitive()))
            })
            .collect())
    }

    fn get_price_alerts_for_device_id(&mut self, device_id: &str, asset_id: Option<&str>) -> Result<Vec<DevicePriceAlert>, DatabaseError> {
        let results = PriceAlertsStore::get_price_alerts_for_device_id(self, device_id, asset_id)?;
        Ok(results
            .into_iter()
            .map(|(alert, device)| DevicePriceAlert {
                device: device.as_primitive(),
                price_alert: alert.as_primitive(),
            })
            .collect())
    }

    fn add_price_alerts(&mut self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, DatabaseError> {
        let device = DevicesStore::get_device(self, device_id).or_not_found(device_id.to_string())?;
        let values = price_alerts.into_iter().map(|x| crate::models::PriceAlertRow::new_price_alert(x, device.id)).collect();
        Ok(PriceAlertsStore::add_price_alerts(self, values)?)
    }

    fn delete_price_alerts(&mut self, device_id: &str, ids: Vec<String>) -> Result<usize, DatabaseError> {
        let device = DevicesStore::get_device(self, device_id).or_not_found(device_id.to_string())?;
        Ok(PriceAlertsStore::delete_price_alerts(self, device.id, ids)?)
    }

    fn update_price_alerts_set_notified_at(&mut self, ids: Vec<String>, last_notified_at: NaiveDateTime) -> Result<usize, DatabaseError> {
        Ok(PriceAlertsStore::update_price_alerts_set_notified_at(self, ids, last_notified_at)?)
    }
}
