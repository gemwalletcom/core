use primitives::{AssetId, AssetIdVecExt, AssetVecExt, Chain};
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

use async_trait::async_trait;
use settings_chain::ChainProviders;
use storage::{models::AssetAddress, AssetsAddressesStore, DatabaseClient};
use streamer::{consumer::MessageConsumer, ChainAddressPayload, StreamProducer, StreamProducerQueue};

pub struct FetchAssetsAddressesConsumer {
    pub provider: ChainProviders,
    pub database: Arc<Mutex<DatabaseClient>>,
    pub stream_producer: StreamProducer,
}

impl FetchAssetsAddressesConsumer {
    pub fn new(provider: ChainProviders, database: Arc<Mutex<DatabaseClient>>, stream_producer: StreamProducer) -> Self {
        Self {
            provider,
            database,
            stream_producer,
        }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchAssetsAddressesConsumer {
    async fn process(&mut self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chains = [Chain::Solana, Chain::Sui];

        for value in payload.values.clone() {
            if !chains.contains(&value.chain) {
                continue;
            }
            let assets = self
                .provider
                .get_assets_balances(value.chain, value.address.clone())
                .await?
                .into_iter()
                .filter(|x| x.balance != "0")
                .collect::<Vec<_>>();

            let assets_addresses = assets
                .into_iter()
                .map(|x| AssetAddress::new(value.chain.to_string(), x.asset_id.to_string(), value.address.clone()))
                .collect::<Vec<_>>();

            let assets_ids = assets_addresses.iter().flat_map(|x| AssetId::new(&x.asset_id.clone())).collect::<Vec<_>>();
            let existing_asset_ids = self
                .database
                .lock()
                .await
                .get_assets(assets_ids.ids().clone())?
                .into_iter()
                .map(|x| x.as_primitive())
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

            self.database.lock().await.add_assets_addresses(results.clone())?;

            self.stream_producer.publish_fetch_assets(missing_asset_ids).await?;
        }

        Ok(payload.values.len())
    }
}
