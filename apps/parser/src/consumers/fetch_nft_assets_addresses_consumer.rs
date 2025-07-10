use std::{collections::HashMap, error::Error, sync::Arc};
use tokio::sync::Mutex;

use async_trait::async_trait;
use cacher::CacherClient;
use storage::DatabaseClient;
use streamer::{consumer::MessageConsumer, ChainAddressPayload, StreamProducer};

pub struct FetchNftAssetsAddressesConsumer {
    pub database: Arc<Mutex<DatabaseClient>>,
    pub stream_producer: StreamProducer,
    pub cacher: CacherClient,
}

impl FetchNftAssetsAddressesConsumer {
    pub fn new(database: Arc<Mutex<DatabaseClient>>, stream_producer: StreamProducer, cacher: CacherClient) -> Self {
        Self {
            database,
            stream_producer,
            cacher,
        }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchNftAssetsAddressesConsumer {
    async fn should_process(&mut self, _payload: ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        // self.cacher
        //     .can_process_now("fetch_nft_assets_addresses", &payload.value.to_string(), 30 * 86400)
        //     .await
        Ok(true)
    }

    async fn process(&mut self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let _map = HashMap::from([(payload.value.chain, payload.value.address.clone())]);
        //let assets = self.nft.get_assets(map).await?;
        //println!("consumer fetch_nft_assets_mappings result: {assets:?}");
        Ok(0)
    }
}
