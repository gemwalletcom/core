use std::error::Error;
use chrono::NaiveDateTime;

use crate::DatabaseClient;
use crate::database::price_alerts::PriceAlertsStore;
use crate::models::{PriceAlert, Price, NewPriceAlert};

pub trait PriceAlertsRepository {
    fn get_price_alerts(&mut self, after_notified_at: NaiveDateTime) -> Result<Vec<(PriceAlert, Price)>, Box<dyn Error + Send + Sync>>;
    fn get_price_alerts_for_device_id(&mut self, device_id: i32) -> Result<Vec<PriceAlert>, Box<dyn Error + Send + Sync>>;
    fn add_price_alerts(&mut self, values: Vec<NewPriceAlert>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn delete_price_alerts(&mut self, device_id: i32, ids: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn update_price_alerts_set_notified_at(&mut self, ids: Vec<String>, last_notified_at: NaiveDateTime) -> Result<usize, Box<dyn Error + Send + Sync>>;
}

impl PriceAlertsRepository for DatabaseClient {
    fn get_price_alerts(&mut self, after_notified_at: NaiveDateTime) -> Result<Vec<(PriceAlert, Price)>, Box<dyn Error + Send + Sync>> {
        Ok(PriceAlertsStore::get_price_alerts(self, after_notified_at)?)
    }

    fn get_price_alerts_for_device_id(&mut self, device_id: i32) -> Result<Vec<PriceAlert>, Box<dyn Error + Send + Sync>> {
        Ok(PriceAlertsStore::get_price_alerts_for_device_id(self, device_id)?)
    }

    fn add_price_alerts(&mut self, values: Vec<NewPriceAlert>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(PriceAlertsStore::add_price_alerts(self, values)?)
    }

    fn delete_price_alerts(&mut self, device_id: i32, ids: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(PriceAlertsStore::delete_price_alerts(self, device_id, ids)?)
    }

    fn update_price_alerts_set_notified_at(&mut self, ids: Vec<String>, last_notified_at: NaiveDateTime) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(PriceAlertsStore::update_price_alerts_set_notified_at(self, ids, last_notified_at)?)
    }
}