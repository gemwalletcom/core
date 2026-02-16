pub mod fetch_prices_consumer;

use std::error::Error;
use std::sync::Arc;

use cacher::CacherClient;
use coingecko::CoinGeckoClient;
use pricer::PriceClient;
use settings::Settings;
use storage::Database;
use streamer::{ConsumerStatusReporter, FetchPricesPayload, QueueName, ShutdownReceiver, run_consumer};

use crate::consumers::{consumer_config, producer_for_queue, reader_for_queue};
use crate::metrics::price::PriceMetrics;
use crate::worker::prices::price_updater::PriceUpdater;

pub async fn run_consumer_prices(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
    price_metrics: Arc<PriceMetrics>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::FetchPrices;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);
    let price_client = PriceClient::new(database, cacher_client);
    let stream_producer = producer_for_queue(&settings, &name).await?;
    let price_updater = PriceUpdater::new(price_client, coingecko_client, stream_producer, price_metrics);
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
