use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use storage::{AssetsAddressesStore, DatabaseClient};
use streamer::{consumer::MessageConsumer, AddressAssetsPayload};
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
impl MessageConsumer<AddressAssetsPayload, usize> for AssetsAddressesConsumer {
    async fn process(&mut self, payload: AddressAssetsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let assets_addresses = payload
            .values
            .into_iter()
            .map(storage::models::AssetAddress::from_primitive)
            .collect::<Vec<_>>();

        Ok(self.database.lock().await.add_assets_addresses(assets_addresses.clone())?)
    }
}
