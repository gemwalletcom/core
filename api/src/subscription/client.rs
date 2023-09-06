extern crate rocket;
use std::error::Error;

use primitives::Subscription;
use storage::DatabaseClient;

pub struct SubscriptionsClient {
    database: DatabaseClient,
}

impl SubscriptionsClient {
    pub async fn new(
        database_url: &str
    ) -> Self {
        let database = DatabaseClient::new(database_url);
        Self {
            database,
        }
    }

    pub fn add_subscriptions(&mut self, device_id: &str, subscriptions: Vec<Subscription>) -> Result<usize, Box<dyn Error>> {
        let device = self.database.get_device(device_id)?;
        let subscriptions = subscriptions
            .into_iter()
            .map(|x| storage::models::Subscription::from_primitive(x, device.id))
            .collect();
        let result = self.database.add_subscriptions(subscriptions)?;
        
        return Ok(result)
    }

    pub fn get_subscriptions(&mut self, device_id: &str) -> Result<Vec<primitives::Subscription>, Box<dyn Error>> {
        let subscriptions = self.database
            .get_subscriptions_by_device_id(device_id)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect();
        return Ok(subscriptions)
    }

    pub fn delete_subscriptions(&mut self, device_id: &str, subscriptions: Vec<Subscription>) -> Result<usize, Box<dyn Error>> {
        let device = self.database.get_device(device_id)?;
        let values = subscriptions
            .into_iter()
            .map(|x| storage::models::Subscription::from_primitive(x, device.id))
            .collect::<Vec<storage::models::Subscription>>();

        //TODO: Implement to delete all subscriptions at once
        let mut result = 0;
        for subscription in values {
            let size = self.database.delete_subscription(subscription)?;
            result += size;
        }
        Ok(result)
    }
}