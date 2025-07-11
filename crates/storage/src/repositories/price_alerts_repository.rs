use chrono::NaiveDateTime;
use std::error::Error;

use crate::database::devices::DevicesStore;
use crate::database::price_alerts::PriceAlertsStore;
use crate::DatabaseClient;

pub trait PriceAlertsRepository {
    fn get_price_alerts(
        &mut self,
        after_notified_at: NaiveDateTime,
    ) -> Result<Vec<(primitives::PriceAlert, primitives::Price, primitives::Device)>, Box<dyn Error + Send + Sync>>;
    fn get_price_alerts_for_device_id(&mut self, device_id: &str) -> Result<Vec<primitives::DevicePriceAlert>, Box<dyn Error + Send + Sync>>;
    fn add_price_alerts(&mut self, device_id: &str, price_alerts: primitives::PriceAlerts) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn delete_price_alerts(&mut self, device_id: &str, ids: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn update_price_alerts_set_notified_at(&mut self, ids: Vec<String>, last_notified_at: NaiveDateTime) -> Result<usize, Box<dyn Error + Send + Sync>>;
}

impl PriceAlertsRepository for DatabaseClient {
    fn get_price_alerts(
        &mut self,
        after_notified_at: NaiveDateTime,
    ) -> Result<Vec<(primitives::PriceAlert, primitives::Price, primitives::Device)>, Box<dyn Error + Send + Sync>> {
        let results = PriceAlertsStore::get_price_alerts(self, after_notified_at)?;
        Ok(results
            .into_iter()
            .map(|(alert, price, device)| (alert.as_primitive(), price.as_price_primitive(), device.as_primitive()))
            .collect())
    }

    fn get_price_alerts_for_device_id(&mut self, device_id: &str) -> Result<Vec<primitives::DevicePriceAlert>, Box<dyn Error + Send + Sync>> {
        let results = PriceAlertsStore::get_price_alerts_for_device_id(self, device_id)?;
        Ok(results
            .into_iter()
            .map(|(alert, device)| primitives::DevicePriceAlert {
                device: device.as_primitive(),
                price_alert: alert.as_primitive(),
            })
            .collect())
    }

    fn add_price_alerts(&mut self, device_id: &str, price_alerts: primitives::PriceAlerts) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let device = DevicesStore::get_device(self, device_id)?;
        let values = price_alerts
            .into_iter()
            .map(|x| crate::models::PriceAlert::new_price_alert(x, device.id))
            .collect();
        Ok(PriceAlertsStore::add_price_alerts(self, values)?)
    }

    fn delete_price_alerts(&mut self, device_id: &str, ids: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let device = DevicesStore::get_device(self, device_id)?;
        Ok(PriceAlertsStore::delete_price_alerts(self, device.id, ids)?)
    }

    fn update_price_alerts_set_notified_at(&mut self, ids: Vec<String>, last_notified_at: NaiveDateTime) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(PriceAlertsStore::update_price_alerts_set_notified_at(self, ids, last_notified_at)?)
    }
}
