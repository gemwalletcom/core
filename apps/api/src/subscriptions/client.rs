extern crate rocket;
use std::error::Error;

use primitives::Subscription;
use storage::{DatabaseClient, SubscriptionsStore};
use streamer::{ChainAddressPayload, ExchangeName, StreamProducer};

pub struct SubscriptionsClient {
    database: DatabaseClient,
    stream_producer: StreamProducer,
}

impl SubscriptionsClient {
    pub async fn new(database_url: &str, stream_producer: StreamProducer) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database, stream_producer }
    }

    pub async fn add_subscriptions(&mut self, device_id: &str, subscriptions: Vec<Subscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let device = self.database.get_device(device_id)?;
        let subscriptions = subscriptions
            .into_iter()
            .map(|x| storage::models::Subscription::from_primitive(x, device.id))
            .collect::<Vec<_>>();
        let result = self.database.add_subscriptions(subscriptions.clone())?;
        let payload = subscriptions
            .clone()
            .into_iter()
            .map(|x| ChainAddressPayload::new(x.as_chain_address()))
            .collect::<Vec<_>>();
        self.stream_producer.publish_to_exchange_batch(ExchangeName::NewAddresses, &payload).await?;
        Ok(result)
    }

    pub async fn get_subscriptions(&mut self, device_id: &str) -> Result<Vec<primitives::Subscription>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self
            .database
            .get_subscriptions_by_device_id(device_id)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect();
        Ok(subscriptions)
    }

    pub async fn delete_subscriptions(&mut self, device_id: &str, subscriptions: Vec<Subscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let device = self.database.get_device(device_id)?;
        let values = subscriptions
            .into_iter()
            .map(|x| storage::models::Subscription::from_primitive(x, device.id))
            .collect::<Vec<storage::models::Subscription>>();

        Ok(self.database.delete_subscriptions(values)?)
    }
}
