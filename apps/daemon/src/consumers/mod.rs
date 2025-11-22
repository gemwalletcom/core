pub mod assets_addresses_consumer;
pub mod fetch_address_transactions_consumer;
pub mod fetch_assets_consumer;
pub mod fetch_blocks_consumer;
pub mod fetch_coin_addresses_consumer;
pub mod fetch_nft_assets_addresses_consumer;
pub mod fetch_token_addresses_consumer;
pub mod notifications;
pub mod store_charts_consumer;
pub mod store_prices_consumer;
pub mod store_transactions_consumer;
pub mod store_transactions_consumer_config;
pub mod support;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

use ::nft::{NFTClient, NFTProviderConfig};
pub use assets_addresses_consumer::AssetsAddressesConsumer;
use cacher::CacherClient;
pub use fetch_assets_consumer::FetchAssetsConsumer;
use pricer::PriceClient;
use settings::Settings;
use settings_chain::ChainProviders;
use storage::Database;
pub use store_charts_consumer::StoreChartsConsumer;
pub use store_prices_consumer::StorePricesConsumer;
pub use store_transactions_consumer::StoreTransactionsConsumer;
pub use store_transactions_consumer_config::StoreTransactionsConsumerConfig;
use streamer::{
    AssetsAddressPayload, ChainAddressPayload, ChartsPayload, ConsumerConfig, FetchAssetsPayload, FetchBlocksPayload, FiatWebhookPayload, PricesPayload,
    QueueName, StreamProducer, StreamReader, StreamReaderConfig, SupportWebhookPayload, TransactionsPayload,
};

use crate::{
    consumers::{
        fetch_address_transactions_consumer::FetchAddressTransactionsConsumer, fetch_blocks_consumer::FetchBlocksConsumer,
        fetch_coin_addresses_consumer::FetchCoinAddressesConsumer, fetch_nft_assets_addresses_consumer::FetchNftAssetsAddressesConsumer,
        fetch_token_addresses_consumer::FetchTokenAddressesConsumer,
    },
    pusher::Pusher,
};
use settings::service_user_agent;

pub async fn run_consumer_fetch_assets(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchAssets;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let consumer = FetchAssetsConsumer {
        providers: chain_providers(&settings, &name),
        database,
        cacher,
    };
    streamer::run_consumer::<FetchAssetsPayload, FetchAssetsConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

pub async fn run_consumer_store_transactions(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::StoreTransactions;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let pusher = Pusher::new(database.clone());

    let consumer = StoreTransactionsConsumer {
        database,
        stream_producer,
        pusher,
        config: StoreTransactionsConsumerConfig {
            min_transaction_amount_usd: settings.daemon.transactions.amount.min,
        },
    };
    streamer::run_consumer::<TransactionsPayload, StoreTransactionsConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

pub fn chain_providers(settings: &Settings, name: &str) -> ChainProviders {
    ChainProviders::from_settings(settings, &service_user_agent("consumer", Some(name)))
}

pub async fn run_consumer_fetch_address_transactions(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchAddressTransactions;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let consumer = FetchAddressTransactionsConsumer::new(database, chain_providers(&settings, &name), stream_producer, cacher);
    streamer::run_consumer::<ChainAddressPayload, FetchAddressTransactionsConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default())
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

pub async fn run_consumer_store_assets_associations(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::StoreAssetsAssociations;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let consumer = AssetsAddressesConsumer::new(database);
    streamer::run_consumer::<AssetsAddressPayload, AssetsAddressesConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

pub async fn run_consumer_fetch_token_associations(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchTokenAssociations;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let consumer = FetchTokenAddressesConsumer::new(chain_providers(&settings, &name), database, stream_producer, cacher);
    streamer::run_consumer::<ChainAddressPayload, FetchTokenAddressesConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

pub async fn run_consumer_fetch_coin_associations(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchCoinAssociations;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let consumer = FetchCoinAddressesConsumer::new(chain_providers(&settings, &name), database, cacher);
    streamer::run_consumer::<ChainAddressPayload, FetchCoinAddressesConsumer, String>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

pub async fn run_consumer_fetch_nft_associations(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchNftAssociations;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let nft_config = NFTProviderConfig::new(settings.nft.opensea.key.secret.clone(), settings.nft.magiceden.key.secret.clone());
    let nft_client = NFTClient::new(database.clone(), nft_config);
    let nft_client = Arc::new(Mutex::new(nft_client));
    let consumer = FetchNftAssetsAddressesConsumer::new(database, stream_producer, cacher, nft_client);
    streamer::run_consumer::<ChainAddressPayload, FetchNftAssetsAddressesConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default())
        .await
}

pub async fn run_consumer_support(settings: Settings, _database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    use support::support_webhook_consumer::SupportWebhookConsumer;
    let queue = QueueName::SupportWebhooks;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let consumer = SupportWebhookConsumer::new(&settings).await?;
    streamer::run_consumer::<SupportWebhookPayload, SupportWebhookConsumer, bool>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

pub async fn run_consumer_fiat(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    use crate::worker::fiat::fiat_webhook_consumer::FiatWebhookConsumer;
    let queue = QueueName::FiatOrderWebhooks;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let consumer = FiatWebhookConsumer::new(database, settings.clone());
    streamer::run_consumer::<FiatWebhookPayload, FiatWebhookConsumer, bool>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

pub async fn run_consumer_store_prices(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::StorePrices;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let consumer = StorePricesConsumer::new(database);
    streamer::run_consumer::<PricesPayload, StorePricesConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

pub async fn run_consumer_store_charts(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::StoreCharts;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let price_client = PriceClient::new(database, cacher_client);
    let consumer = StoreChartsConsumer::new(price_client);
    streamer::run_consumer::<ChartsPayload, StoreChartsConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}
