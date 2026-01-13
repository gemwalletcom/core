use std::error::Error;

use primitives::{ChainAddress, Subscription};
use storage::{Database, WalletsRepository};
use streamer::{ChainAddressPayload, StreamProducer, StreamProducerQueue};

#[derive(Clone)]
pub struct WalletsClient {
    database: Database,
    stream_producer: StreamProducer,
}

impl WalletsClient {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub async fn get_wallet_subscriptions(&self, device_id: i32, wallet_id: &str) -> Result<Vec<Subscription>, Box<dyn Error + Send + Sync>> {
        let mut client = self.database.wallets()?;
        let subscriptions = client.get_wallet_subscriptions(wallet_id, device_id)?;

        Ok(subscriptions
            .into_iter()
            .filter_map(|s| {
                s.chain.parse().ok().map(|chain| Subscription {
                    wallet_index: s.wallet_index,
                    chain,
                    address: s.address,
                })
            })
            .collect())
    }

    pub async fn add_wallet_subscriptions(&self, device_id: i32, wallet_id: &str, subscriptions: Vec<Subscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut client = self.database.wallets()?;

        let subs: Vec<(String, String, i32)> = subscriptions
            .iter()
            .map(|s| (s.chain.to_string(), s.address.clone(), s.wallet_index))
            .collect();

        let count = client.add_wallet_subscriptions(wallet_id, device_id, subs)?;

        let payload = subscriptions
            .into_iter()
            .map(|x| ChainAddressPayload::new(ChainAddress::new(x.chain, x.address)))
            .collect::<Vec<_>>();

        self.stream_producer.publish_new_addresses(payload).await?;

        Ok(count)
    }

    pub async fn delete_wallet_subscriptions(&self, device_id: i32, wallet_id: &str, subscriptions: Vec<Subscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut client = self.database.wallets()?;

        let subs: Vec<(String, String, i32)> = subscriptions
            .iter()
            .map(|s| (s.chain.to_string(), s.address.clone(), s.wallet_index))
            .collect();

        Ok(client.delete_wallet_subscriptions(wallet_id, device_id, subs)?)
    }
}
