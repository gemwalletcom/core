use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use storage::DatabaseClient;
use streamer::{AssetsAddressPayload, consumer::MessageConsumer};
use tokio::sync::Mutex;

pub struct AssetsAddressesConsumer {
    pub database: Arc<Mutex<DatabaseClient>>,
}

impl AssetsAddressesConsumer {
    pub fn new(database: Arc<Mutex<DatabaseClient>>) -> Self {
        Self { database }
    }
}

#[async_trait]
impl MessageConsumer<AssetsAddressPayload, usize> for AssetsAddressesConsumer {
    async fn should_process(&mut self, _payload: AssetsAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&mut self, payload: AssetsAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let assets_addresses = payload
            .values
            .into_iter()
            .map(storage::models::AssetAddress::from_primitive)
            .collect::<Vec<_>>();

        Ok(self
            .database
            .lock()
            .await
            .assets_addresses()
            .add_assets_addresses(assets_addresses.clone().into_iter().map(|x| x.as_primitive()).collect())?)
    }
}
