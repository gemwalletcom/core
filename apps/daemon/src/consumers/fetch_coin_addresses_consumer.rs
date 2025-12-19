use num_bigint::BigUint;
use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use settings_chain::ChainProviders;
use storage::Database;
use storage::models::AssetAddressRow;
use streamer::{ChainAddressPayload, consumer::MessageConsumer};

pub struct FetchCoinAddressesConsumer {
    pub provider: ChainProviders,
    pub database: Database,
    pub cacher: CacherClient,
}

impl FetchCoinAddressesConsumer {
    pub fn new(provider: ChainProviders, database: Database, cacher: CacherClient) -> Self {
        Self { provider, database, cacher }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, String> for FetchCoinAddressesConsumer {
    async fn should_process(&self, payload: ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher
            .can_process_cached(CacheKey::FetchCoinAddresses(payload.value.chain.as_ref(), &payload.value.address))
            .await
    }

    async fn process(&self, payload: ChainAddressPayload) -> Result<String, Box<dyn Error + Send + Sync>> {
        let balance = self.provider.get_balance_coin(payload.value.chain, payload.value.address.clone()).await?;

        if balance.balance.available == BigUint::ZERO {
            return Ok(balance.balance.available.to_string());
        }

        let asset_address = AssetAddressRow::new(
            payload.value.chain.to_string(),
            balance.asset_id.to_string(),
            payload.value.address,
            Some(balance.balance.available.to_string()),
        );

        let _ = self
            .database
            .client()?
            .assets_addresses()
            .add_assets_addresses(vec![asset_address.as_primitive()])?;

        Ok(balance.balance.available.to_string())
    }
}
