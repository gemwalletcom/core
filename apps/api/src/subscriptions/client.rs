use std::error::Error;

use primitives::{ChainAddress, Subscription};
use storage::DatabaseClient;
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
        let result = self.database.subscriptions().add_subscriptions(subscriptions.clone(), device_id);
        let payload = subscriptions
            .clone()
            .into_iter()
            .map(|x| ChainAddressPayload::new(ChainAddress::new(x.chain, x.address)))
            .collect::<Vec<_>>();
        self.stream_producer.publish_to_exchange_batch(ExchangeName::NewAddresses, &payload).await?;
        Ok(result?)
    }

    pub async fn get_subscriptions_by_device_id(&mut self, device_id: &str) -> Result<Vec<Subscription>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.subscriptions().get_subscriptions_by_device_id(device_id, None)?)
    }

    pub async fn delete_subscriptions(&mut self, device_id: &str, subscriptions: Vec<Subscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.subscriptions().delete_subscriptions(subscriptions, device_id)?)
    }
}
