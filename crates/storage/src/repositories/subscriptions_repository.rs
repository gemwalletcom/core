use std::error::Error;

use crate::{models::Subscription, DatabaseClient};

pub struct SubscriptionsRepository<'a> {
    database: &'a mut DatabaseClient,
}

impl<'a> SubscriptionsRepository<'a> {
    pub fn new(database: &'a mut DatabaseClient) -> Self {
        Self { database }
    }

    pub fn get_subscriptions_by_device_id_wallet_index(
        &mut self,
        device_id: &str,
        wallet_index: i32,
    ) -> Result<Vec<Subscription>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.get_subscriptions_by_device_id_wallet_index(device_id, wallet_index)?)
    }

    pub fn get_subscriptions_by_device_id(&mut self, device_id: i32) -> Result<Vec<Subscription>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.get_subscriptions_by_device_id(device_id)?)
    }

    pub fn get_subscriptions_by_device_id_str(&mut self, device_id: &str) -> Result<Vec<Subscription>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.get_subscriptions_by_device_id_str(device_id)?)
    }

    pub fn add_subscriptions(&mut self, values: Vec<Subscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.add_subscriptions(values)?)
    }

    pub fn delete_subscriptions(&mut self, values: Vec<Subscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.delete_subscriptions(values)?)
    }
}