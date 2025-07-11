use primitives::{AssetId, AssetIdVecExt, AssetVecExt, Chain};
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

use async_trait::async_trait;
use cacher::CacherClient;
use settings_chain::ChainProviders;
use storage::{models::AssetAddress, DatabaseClient};
use streamer::{consumer::MessageConsumer, ChainAddressPayload, StreamProducer, StreamProducerQueue};

pub struct FetchAssetsAddressesConsumer {
    pub provider: ChainProviders,
    pub database: Arc<Mutex<DatabaseClient>>,
    pub stream_producer: StreamProducer,
    pub cacher: CacherClient,
}

#[derive(Clone)]
pub struct FetchAssetsAddressesResult {
    assets: Vec<AssetAddress>,
    zero_balance_assets: Vec<AssetAddress>,
    missing_asset_ids: Vec<AssetId>,
}

impl FetchAssetsAddressesConsumer {
    pub fn new(provider: ChainProviders, database: Arc<Mutex<DatabaseClient>>, stream_producer: StreamProducer, cacher: CacherClient) -> Self {
        Self {
            provider,
            database,
            stream_producer,
            cacher,
        }
    }

    async fn fetch_assets_addresses(&self, chain: Chain, address: String) -> Result<FetchAssetsAddressesResult, Box<dyn Error + Send + Sync>> {
        let assets = self.provider.get_assets_balances(chain, address.clone()).await?;

        let assets = assets.clone().into_iter().filter(|x| x.balance != "0").collect::<Vec<_>>();
        let zero_balance_assets = assets
            .clone()
            .into_iter()
            .filter(|x| x.balance == "0")
            .map(|x| AssetAddress::new(x.asset_id.chain.to_string(), x.asset_id.to_string(), address.clone()))
            .collect::<Vec<_>>();

        let assets_addresses = assets
            .into_iter()
            .map(|x| AssetAddress::new(chain.to_string(), x.asset_id.to_string(), address.clone()))
            .collect::<Vec<_>>();

        let assets_ids = assets_addresses.iter().flat_map(|x| AssetId::new(&x.asset_id.clone())).collect::<Vec<_>>();
        let existing_asset_ids = self
            .database
            .lock()
            .await
            .assets()
            .get_assets(assets_ids.ids().clone())?
            .into_iter()
            .collect::<Vec<_>>()
            .ids();

        let missing_asset_ids = assets_ids
            .clone()
            .into_iter()
            .filter(|x| !existing_asset_ids.iter().any(|a| a == x))
            .collect::<Vec<_>>();

        let results = assets_addresses
            .into_iter()
            .filter(|x| existing_asset_ids.iter().any(|a| x.asset_id == a.to_string()))
            .collect::<Vec<_>>();

        Ok(FetchAssetsAddressesResult {
            assets: results,
            zero_balance_assets,
            missing_asset_ids,
        })
    }

    async fn process_result(&self, result: FetchAssetsAddressesResult) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if !result.zero_balance_assets.is_empty() {
            let _ = self
                .database
                .lock()
                .await
                .assets_addresses()
                .delete_assets_addresses(result.zero_balance_assets.clone().into_iter().map(|x| x.as_primitive()).collect());
        }
        if !result.assets.is_empty() {
            let _ = self
                .database
                .lock()
                .await
                .assets_addresses()
                .add_assets_addresses(result.assets.clone().into_iter().map(|x| x.as_primitive()).collect());
        }
        self.stream_producer.publish_fetch_assets(result.missing_asset_ids.clone()).await
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchAssetsAddressesConsumer {
    async fn should_process(&mut self, payload: ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher
            .can_process_now("fetch_assets_addresses", &payload.value.to_string(), 30 * 86400)
            .await
    }

    async fn process(&mut self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let result = self.fetch_assets_addresses(payload.value.chain, payload.value.address.clone()).await?;
        let _ = self.process_result(result.clone()).await;
        Ok(result.assets.len())
    }
}
