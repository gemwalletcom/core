use std::{collections::HashMap, error::Error, sync::Arc};
use tokio::sync::Mutex;


use ::nft::NFTClient;
use async_trait::async_trait;
use cacher::CacherClient;
use storage::Database;
use streamer::{ChainAddressPayload, StreamProducer, consumer::MessageConsumer};

pub struct FetchNftAssetsAddressesConsumer {
    #[allow(dead_code)]
    pub database: Database,
    #[allow(dead_code)]
    pub stream_producer: StreamProducer,
    pub cacher: CacherClient,
    pub nft_client: Arc<Mutex<NFTClient>>,
}

impl FetchNftAssetsAddressesConsumer {
    pub fn new(database: Database, stream_producer: StreamProducer, cacher: CacherClient, nft_client: Arc<Mutex<NFTClient>>) -> Self {
        Self {
            database,
            stream_producer,
            cacher,
            nft_client,
        }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchNftAssetsAddressesConsumer {
    async fn should_process(&mut self, payload: ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher
            .can_process_now(
                &format!("fetch_nft_assets_addresses:{}:{}", payload.value.chain, payload.value.address),
                30 * 86400,
            )
            .await
    }

    async fn process(&mut self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let map = HashMap::from([(payload.value.chain, payload.value.address.clone())]);
        let assets = self.nft_client.lock().await.fetch_assets_for_addresses(map).await?;
        Ok(assets.len())
    }
}
