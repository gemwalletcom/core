use std::{collections::HashMap, error::Error, sync::Arc};
use tokio::sync::Mutex;

use ::nft::{NFTClient, NFTProviderConfig};
use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use primitives::Chain;
use settings::Settings;
use storage::Database;
use streamer::{ChainAddressPayload, ConsumerConfig, QueueName, ShutdownReceiver, StreamConnection, StreamProducer, StreamReader, consumer::MessageConsumer, run_consumer};

pub struct FetchNftAssetsAddressesConsumer {
    #[allow(dead_code)]
    pub database: Database,
    #[allow(dead_code)]
    pub stream_producer: StreamProducer,
    pub cacher: CacherClient,
    pub nft_client: Arc<Mutex<NFTClient>>,
}

impl FetchNftAssetsAddressesConsumer {
    pub async fn run(
        settings: Settings,
        database: Database,
        chain: Chain,
        connection: &StreamConnection,
        cacher: CacherClient,
        consumer_config: ConsumerConfig,
        shutdown_rx: ShutdownReceiver,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let queue = QueueName::FetchNftAssociations;
        let name = format!("{}.{}", queue, chain.as_ref());
        let stream_reader = StreamReader::from_connection(connection, settings.rabbitmq.prefetch).await?;
        let stream_producer = StreamProducer::from_connection(connection).await?;
        let nft_config = NFTProviderConfig::new(settings.nft.opensea.key.secret.clone(), settings.nft.magiceden.key.secret.clone());
        let nft_client = NFTClient::new(database.clone(), nft_config);
        let nft_client = Arc::new(Mutex::new(nft_client));
        let consumer = Self {
            database,
            stream_producer,
            cacher,
            nft_client,
        };
        run_consumer::<ChainAddressPayload, Self, usize>(&name, stream_reader, queue, Some(chain.as_ref()), consumer, consumer_config, shutdown_rx).await
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchNftAssetsAddressesConsumer {
    async fn should_process(&self, payload: ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher
            .can_process_cached(CacheKey::FetchNftAssetsAddresses(payload.value.chain.as_ref(), &payload.value.address))
            .await
    }

    async fn process(&self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let map = HashMap::from([(payload.value.chain, payload.value.address.clone())]);
        let assets = self.nft_client.lock().await.fetch_assets_for_addresses(map).await?;
        Ok(assets.len())
    }
}
