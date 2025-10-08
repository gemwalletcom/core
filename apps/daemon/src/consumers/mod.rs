pub mod assets_addresses_consumer;
pub mod fetch_address_transactions_consumer;
pub mod fetch_assets_consumer;
pub mod fetch_blocks_consumer;
pub mod fetch_coin_addresses_consumer;
pub mod fetch_nft_assets_addresses_consumer;
pub mod fetch_token_addresses_consumer;
pub mod notifications;
pub mod store_transactions_consumer;
pub mod store_transactions_consumer_config;
pub mod support;
use std::error::Error;
use std::sync::Arc;

use ::nft::{NFTClient, NFTProviderConfig};
pub use assets_addresses_consumer::AssetsAddressesConsumer;
use cacher::CacherClient;
pub use fetch_assets_consumer::FetchAssetsConsumer;
use settings::Settings;
use settings_chain::ChainProviders;
use storage::DatabaseClient;
pub use store_transactions_consumer::StoreTransactionsConsumer;
pub use store_transactions_consumer_config::StoreTransactionsConsumerConfig;
use streamer::{
    AssetsAddressPayload, ChainAddressPayload, ConsumerConfig, FetchAssetsPayload, FetchBlocksPayload, QueueName, StreamProducer, StreamReader,
    StreamReaderConfig, TransactionsPayload,
};
use tokio::sync::Mutex;

use crate::{
    consumers::{
        fetch_address_transactions_consumer::FetchAddressTransactionsConsumer, fetch_blocks_consumer::FetchBlocksConsumer,
        fetch_coin_addresses_consumer::FetchCoinAddressesConsumer, fetch_nft_assets_addresses_consumer::FetchNftAssetsAddressesConsumer,
        fetch_token_addresses_consumer::FetchTokenAddressesConsumer,
    },
    pusher::Pusher,
};
use settings::service_user_agent;

pub async fn run_consumer_fetch_assets(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    const NAME: &str = "fetch_assets";
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), NAME.to_string(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let cacher = CacherClient::new(&settings.redis.url);
    let consumer = FetchAssetsConsumer {
        providers: chain_providers(&settings, NAME),
        database: database.clone(),
        cacher,
    };
    streamer::run_consumer::<FetchAssetsPayload, FetchAssetsConsumer, usize>(NAME, stream_reader, QueueName::FetchAssets, consumer, ConsumerConfig::default())
        .await
}

pub async fn run_consumer_store_transactions(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    const NAME: &str = "store_transactions";
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), NAME.to_string(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, NAME).await?;
    let pusher = Pusher::new(&settings.postgres.url);

    let consumer = StoreTransactionsConsumer {
        database: database.clone(),
        stream_producer,
        pusher,
        config: StoreTransactionsConsumerConfig::default(),
    };
    streamer::run_consumer::<TransactionsPayload, StoreTransactionsConsumer, usize>(
        NAME,
        stream_reader,
        QueueName::StoreTransactions,
        consumer,
        ConsumerConfig::default(),
    )
    .await
}

pub fn chain_providers(settings: &Settings, name: &str) -> ChainProviders {
    ChainProviders::from_settings(settings, &service_user_agent("consumer", Some(name)))
}

pub async fn run_consumer_fetch_address_transactions(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    const NAME: &str = "fetch_address_transactions";
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), NAME.to_string(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, NAME).await?;
    let cacher = CacherClient::new(&settings.redis.url);
    let consumer = FetchAddressTransactionsConsumer::new(database.clone(), chain_providers(&settings, NAME), stream_producer, cacher);
    streamer::run_consumer::<ChainAddressPayload, FetchAddressTransactionsConsumer, usize>(
        NAME,
        stream_reader,
        QueueName::FetchAddressTransactions,
        consumer,
        ConsumerConfig::default(),
    )
    .await
}

pub async fn run_consumer_fetch_blocks(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchBlocks;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let consumer = FetchBlocksConsumer::new(chain_providers(&settings, &name), stream_producer);
    streamer::run_consumer::<FetchBlocksPayload, FetchBlocksConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

pub async fn run_consumer_store_assets_associations(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::StoreAssetsAssociations;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let consumer = AssetsAddressesConsumer::new(database.clone());
    streamer::run_consumer::<AssetsAddressPayload, AssetsAddressesConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

pub async fn run_consumer_fetch_token_associations(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchTokenAssociations;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let cacher = CacherClient::new(&settings.redis.url);
    let consumer = FetchTokenAddressesConsumer::new(chain_providers(&settings, &name), database.clone(), stream_producer, cacher);
    streamer::run_consumer::<ChainAddressPayload, FetchTokenAddressesConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

pub async fn run_consumer_fetch_coin_associations(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchCoinAssociations;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let cacher = CacherClient::new(&settings.redis.url);
    let consumer = FetchCoinAddressesConsumer::new(chain_providers(&settings, &name), database.clone(), cacher);
    streamer::run_consumer::<ChainAddressPayload, FetchCoinAddressesConsumer, String>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

pub async fn run_consumer_fetch_nft_associations(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchNftAssociations;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let cacher = CacherClient::new(&settings.redis.url);
    let nft_config = NFTProviderConfig::new(settings.nft.opensea.key.secret.clone(), settings.nft.magiceden.key.secret.clone());
    let nft_client = NFTClient::new(&settings.postgres.url, nft_config).await;
    let nft_client = Arc::new(Mutex::new(nft_client));
    let consumer = FetchNftAssetsAddressesConsumer::new(database.clone(), stream_producer, cacher, nft_client);
    streamer::run_consumer::<ChainAddressPayload, FetchNftAssetsAddressesConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default())
        .await
}

pub async fn run_consumer_support(settings: Settings, _database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    use support::support_webhook_consumer::SupportWebhookConsumer;
    const NAME: &str = "support";
    let consumer = SupportWebhookConsumer::new(&settings).await?;
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), NAME.to_string(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    streamer::run_consumer(NAME, stream_reader, QueueName::SupportWebhooks, consumer, ConsumerConfig::default()).await
}

pub async fn run_consumer_fiat(settings: Settings, _database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    use crate::worker::fiat::fiat_webhook_consumer::FiatWebhookConsumer;
    const NAME: &str = "fiat";
    let consumer = FiatWebhookConsumer::new(&settings.postgres.url, settings.clone());
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), NAME.to_string(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    streamer::run_consumer(NAME, stream_reader, QueueName::FiatOrderWebhooks, consumer, ConsumerConfig::default()).await
}
