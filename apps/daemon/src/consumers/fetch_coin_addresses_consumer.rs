use num_bigint::BigUint;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

use async_trait::async_trait;
use cacher::CacherClient;
use settings_chain::ChainProviders;
use storage::{DatabaseClient, models::AssetAddress};
use streamer::{ChainAddressPayload, consumer::MessageConsumer};

pub struct FetchCoinAddressesConsumer {
    pub provider: ChainProviders,
    pub database: Arc<Mutex<DatabaseClient>>,
    pub cacher: CacherClient,
}

impl FetchCoinAddressesConsumer {
    pub fn new(provider: ChainProviders, database: Arc<Mutex<DatabaseClient>>, cacher: CacherClient) -> Self {
        Self { provider, database, cacher }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, String> for FetchCoinAddressesConsumer {
    async fn should_process(&mut self, payload: ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher
            .can_process_now(&format!("fetch_coin_addresses:{}:{}", payload.value.chain, payload.value.address), 7 * 86400)
            .await
    }

    async fn process(&mut self, payload: ChainAddressPayload) -> Result<String, Box<dyn Error + Send + Sync>> {
        let balance = self.provider.get_balance_coin(payload.value.chain, payload.value.address.clone()).await?;

        if balance.balance.available == BigUint::ZERO {
            return Ok(balance.balance.available.to_string());
        }

        let asset_address = AssetAddress::new(
            payload.value.chain.to_string(),
            balance.asset_id.to_string(),
            payload.value.address,
            Some(balance.balance.available.to_string()),
        );

        let _ = self
            .database
            .lock()
            .await
            .assets_addresses()
            .add_assets_addresses(vec![asset_address.as_primitive()])?;

        Ok(balance.balance.available.to_string())
    }
}
