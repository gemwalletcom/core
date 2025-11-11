use std::error::Error;

use primitives::{ChainAddress, Subscription};
use storage::Database;
use streamer::{ChainAddressPayload, ExchangeName, StreamProducer};

#[derive(Clone)]
pub struct SubscriptionsClient {
    database: Database,
    stream_producer: StreamProducer,
}

impl SubscriptionsClient {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub async fn add_subscriptions(&self, device_id: &str, subscriptions: Vec<Subscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let result = self.database.client()?.subscriptions().add_subscriptions(subscriptions.clone(), device_id);
        let payload = subscriptions
            .clone()
            .into_iter()
            .map(|x| ChainAddressPayload::new(ChainAddress::new(x.chain, x.address)))
            .collect::<Vec<_>>();
        self.stream_producer.publish_to_exchange_batch(ExchangeName::NewAddresses, &payload).await?;
        Ok(result?)
    }

    pub async fn get_subscriptions_by_device_id(&self, device_id: &str) -> Result<Vec<Subscription>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.subscriptions().get_subscriptions_by_device_id(device_id, None)?)
    }

    pub async fn delete_subscriptions(&self, device_id: &str, subscriptions: Vec<Subscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.subscriptions().delete_subscriptions(subscriptions, device_id)?)
    }
}
