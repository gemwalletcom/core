pub mod assets_addresses_consumer;
pub mod fetch_assets_addresses_consumer;
pub mod fetch_assets_consumer;
pub mod fetch_blocks_consumer;
pub mod fetch_transactions_consumer;
pub mod transactions_consumer;
pub mod transactions_consumer_config;
use std::error::Error;
use std::sync::Arc;

pub use assets_addresses_consumer::AssetsAddressesConsumer;
use cacher::CacherClient;
pub use fetch_assets_consumer::FetchAssetsConsumer;
use settings::Settings;
use settings_chain::ChainProviders;
use storage::DatabaseClient;
use streamer::{
    AssetsAddressPayload, ChainAddressPayload, ConsumerConfig, FetchAssetsPayload, FetchBlocksPayload, QueueName, StreamProducer, StreamReader,
    TransactionsPayload,
};
use tokio::sync::Mutex;
pub use transactions_consumer::TransactionsConsumer;
pub use transactions_consumer_config::TransactionsConsumerConfig;

use crate::{
    consumers::{
        fetch_assets_addresses_consumer::FetchAssetsAddressesConsumer, fetch_blocks_consumer::FetchBlocksConsumer,
        fetch_transactions_consumer::FetchTransactionsConsumer,
    },
    Pusher,
};

pub async fn run_consumers(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::spawn(run_consumer_fetch_assets(settings.clone(), database.clone()));
    tokio::spawn(run_consumer_store_transactions(settings.clone(), database.clone()));
    tokio::spawn(run_consumer_fetch_transactions(settings.clone(), database.clone()));
    tokio::spawn(run_consumer_fetch_assets_addresses_associations(settings.clone(), database.clone()));
    tokio::spawn(run_consumer_store_assets_addresses_associations(settings.clone(), database.clone()));
    tokio::spawn(run_consumer_fetch_blocks(settings.clone()));
    std::future::pending::<()>().await;
    Ok(())
}

pub async fn run_consumer_fetch_assets(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = "fetch_assets";
    let stream_reader = StreamReader::new(&settings.rabbitmq.url, name).await?;
    let cacher = CacherClient::new(&settings.redis.url);
    let consumer = FetchAssetsConsumer {
        providers: ChainProviders::from_settings(&settings),
        database: database.clone(),
        cacher,
    };
    streamer::run_consumer::<FetchAssetsPayload, FetchAssetsConsumer, usize>(name, stream_reader, QueueName::FetchAssets, consumer, ConsumerConfig::default())
        .await
}

pub async fn run_consumer_store_transactions(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = "store_transactions";
    let stream_reader = StreamReader::new(&settings.rabbitmq.url, name).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, name).await?;
    let pusher = Pusher::new(&settings.postgres.url);

    let consumer = TransactionsConsumer {
        database: database.clone(),
        stream_producer,
        pusher,
        config: TransactionsConsumerConfig::default(),
    };
    streamer::run_consumer::<TransactionsPayload, TransactionsConsumer, usize>(
        name,
        stream_reader,
        QueueName::StoreTransactions,
        consumer,
        ConsumerConfig::default(),
    )
    .await
}

pub async fn run_consumer_fetch_transactions(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = "fetch_transactions";
    let stream_reader = StreamReader::new(&settings.rabbitmq.url, name).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, name).await?;
    let cacher = CacherClient::new(&settings.redis.url);
    let consumer = FetchTransactionsConsumer::new(database.clone(), ChainProviders::from_settings(&settings), stream_producer, cacher);
    streamer::run_consumer::<ChainAddressPayload, FetchTransactionsConsumer, usize>(
        name,
        stream_reader,
        QueueName::FetchTransactions,
        consumer,
        ConsumerConfig::default(),
    )
    .await
}

pub async fn run_consumer_fetch_blocks(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = "fetch_blocks";
    let stream_reader = StreamReader::new(&settings.rabbitmq.url, name).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, name).await?;
    let consumer = FetchBlocksConsumer::new(ChainProviders::from_settings(&settings), stream_producer);
    streamer::run_consumer::<FetchBlocksPayload, FetchBlocksConsumer, usize>(name, stream_reader, QueueName::FetchBlocks, consumer, ConsumerConfig::default())
        .await
}

pub async fn run_consumer_store_assets_addresses_associations(
    settings: Settings,
    database: Arc<Mutex<DatabaseClient>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = "store_assets_addresses_associations";
    let stream_reader = StreamReader::new(&settings.rabbitmq.url, name).await?;
    let consumer = AssetsAddressesConsumer::new(database.clone());
    streamer::run_consumer::<AssetsAddressPayload, AssetsAddressesConsumer, usize>(
        name,
        stream_reader,
        QueueName::AssetsAddressesAssociations,
        consumer,
        ConsumerConfig::default(),
    )
    .await
}

pub async fn run_consumer_fetch_assets_addresses_associations(
    settings: Settings,
    database: Arc<Mutex<DatabaseClient>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = "fetch_assets_addresses_associations";
    let stream_reader = StreamReader::new(&settings.rabbitmq.url, name).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, name).await?;
    let cacher = CacherClient::new(&settings.redis.url);
    let consumer = FetchAssetsAddressesConsumer::new(ChainProviders::from_settings(&settings), database.clone(), stream_producer, cacher);
    streamer::run_consumer::<ChainAddressPayload, FetchAssetsAddressesConsumer, usize>(
        name,
        stream_reader,
        QueueName::FetchAssetsAddressesAssociations,
        consumer,
        ConsumerConfig::default(),
    )
    .await
}
