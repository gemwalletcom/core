use num_bigint::BigUint;
use primitives::{AssetId, AssetIdVecExt, AssetVecExt};
use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use settings_chain::ChainProviders;
use storage::{AssetsAddressesRepository, AssetsRepository, Database};
use storage::models::AssetAddressRow;
use streamer::{ChainAddressPayload, StreamProducer, StreamProducerQueue, consumer::MessageConsumer};

pub struct FetchTokenAddressesConsumer {
    pub provider: ChainProviders,
    pub database: Database,
    pub stream_producer: StreamProducer,
    pub cacher: CacherClient,
}

impl FetchTokenAddressesConsumer {
    pub fn new(provider: ChainProviders, database: Database, stream_producer: StreamProducer, cacher: CacherClient) -> Self {
        Self {
            provider,
            database,
            stream_producer,
            cacher,
        }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchTokenAddressesConsumer {
    async fn should_process(&self, payload: ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher
            .can_process_cached(CacheKey::FetchTokenAddresses(payload.value.chain.as_ref(), &payload.value.address))
            .await
    }

    async fn process(&self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let all_assets = self.provider.get_balance_assets(payload.value.chain, payload.value.address.clone()).await?;

        let mut zero_balance_addresses = Vec::new();
        let mut non_zero_addresses = Vec::new();

        for asset in all_assets {
            if asset.balance.available == BigUint::ZERO {
                zero_balance_addresses.push(AssetAddressRow::new(
                    asset.asset_id.chain.to_string(),
                    asset.asset_id.to_string(),
                    payload.value.address.clone(),
                    None,
                ));
            } else {
                non_zero_addresses.push(AssetAddressRow::new(
                    payload.value.chain.to_string(),
                    asset.asset_id.to_string(),
                    payload.value.address.clone(),
                    Some(asset.balance.available.to_string()),
                ));
            }
        }

        let asset_ids: Vec<_> = non_zero_addresses.iter().flat_map(|x| AssetId::new(&x.asset_id)).collect();
        let existing_ids = self.database.assets()?.get_assets(asset_ids.ids())?.ids();

        let missing_ids: Vec<_> = asset_ids.into_iter().filter(|id| !existing_ids.contains(id)).collect();
        let existing_addresses: Vec<_> = non_zero_addresses
            .into_iter()
            .filter(|addr| AssetId::new(&addr.asset_id).is_some_and(|id| existing_ids.contains(&id)))
            .collect();

        let _ = self
            .database
            .assets_addresses()?
            .delete_assets_addresses(zero_balance_addresses.into_iter().map(|x| x.as_primitive()).collect());
        let _ = self
            .database
            .assets_addresses()?
            .add_assets_addresses(existing_addresses.iter().map(|x| x.as_primitive()).collect());

        self.stream_producer.publish_fetch_assets(missing_ids).await?;

        Ok(existing_addresses.len())
    }
}
