pub mod fetch_prices_consumer;
pub mod store_charts_consumer;
pub mod store_prices_consumer;

use std::error::Error;
use std::sync::Arc;

use cacher::CacherClient;
use coingecko::CoinGeckoClient;
use pricer::PriceClient;
use primitives::ConfigKey;
use settings::Settings;
use storage::{ConfigCacher, Database};
use streamer::{ChartsPayload, ConsumerStatusReporter, FetchPricesPayload, PricesPayload, QueueName, ShutdownReceiver, run_consumer};

use crate::consumers::{consumer_config, producer_for_queue, reader_for_queue};
use crate::worker::prices::price_updater::PriceUpdater;

use store_charts_consumer::StoreChartsConsumer;
use store_prices_consumer::StorePricesConsumer;

pub async fn run_consumer_store_prices(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = Arc::new(settings);

    futures::future::try_join_all(vec![
        tokio::spawn(run_store_prices(settings.clone(), shutdown_rx.clone(), reporter.clone())),
        tokio::spawn(run_store_charts(settings.clone(), shutdown_rx.clone(), reporter.clone())),
    ])
    .await?;

    Ok(())
}

async fn run_store_prices(settings: Arc<Settings>, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::StorePrices;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let price_client = PriceClient::new(database.clone(), cacher_client);
    let config = ConfigCacher::new(database.clone());
    let ttl_seconds = config.get_duration(ConfigKey::PriceOutdated)?.as_secs() as i64;
    let consumer = StorePricesConsumer::new(database, price_client, ttl_seconds);
    run_consumer::<PricesPayload, StorePricesConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

async fn run_store_charts(settings: Arc<Settings>, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::StoreCharts;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let price_client = PriceClient::new(database, cacher_client);
    let consumer = StoreChartsConsumer::new(price_client);
    run_consumer::<ChartsPayload, StoreChartsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

pub async fn run_consumer_fetch_prices(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::FetchPrices;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);
    let price_client = PriceClient::new(database, cacher_client);
    let stream_producer = producer_for_queue(&settings, &name).await?;
    let price_updater = PriceUpdater::new(price_client, coingecko_client, stream_producer);
    let consumer = fetch_prices_consumer::FetchPricesConsumer::new(price_updater);
    run_consumer::<FetchPricesPayload, fetch_prices_consumer::FetchPricesConsumer, usize>(
        &name,
        stream_reader,
        queue,
        None,
        consumer,
        consumer_config(&settings.consumer),
        shutdown_rx,
        reporter,
    )
    .await
}
