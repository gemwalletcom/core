use num_bigint::BigUint;
use primitives::AssetAddress;
use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use settings_chain::ChainProviders;
use storage::AssetsAddressesRepository;
use storage::Database;
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
        let chain_address = payload.value;
        let balance = self.provider.get_balance_coin(chain_address.chain, chain_address.address.clone()).await?;
        let balance_value = balance.balance.available.to_string();
        let asset_id = balance.asset_id;
        let asset_address = AssetAddress::new(asset_id.clone(), chain_address.address.clone(), Some(balance_value.clone()));
        let mut assets_addresses = self.database.assets_addresses()?;

        if balance.balance.available == BigUint::ZERO && assets_addresses.get_asset_address(chain_address, asset_id)?.is_some() {
            assets_addresses.delete_assets_addresses(vec![asset_address])?;
        } else {
            assets_addresses.add_assets_addresses(vec![asset_address])?;
        }

        Ok(balance_value)
    }
}
