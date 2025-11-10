use std::error::Error;


use async_trait::async_trait;
use storage::Database;
use streamer::{AssetsAddressPayload, consumer::MessageConsumer};


pub struct AssetsAddressesConsumer {
    pub database: Database,
}

impl AssetsAddressesConsumer {
    pub fn new(database: Database) -> Self {
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
            .client()?.assets_addresses()
            .add_assets_addresses(assets_addresses.clone().into_iter().map(|x| x.as_primitive()).collect())?)
    }
}
