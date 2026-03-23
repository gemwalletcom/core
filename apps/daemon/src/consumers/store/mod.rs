pub mod device_stream_consumer;
pub mod store_charts_consumer;
pub mod store_pending_transactions_consumer;
pub mod store_prices_consumer;
pub mod store_transactions_consumer;
pub mod store_transactions_consumer_config;

pub use store_transactions_consumer::StoreTransactionsConsumer;
pub use store_transactions_consumer_config::StoreTransactionsConsumerConfig;

use std::error::Error;
use std::sync::Arc;

use crate::client::SwapVaultAddressClient;
use cacher::CacherClient;
use pricer::PriceClient;
use primitives::{ConfigKey, TransactionId};
use settings::Settings;
use storage::{ConfigCacher, Database};
use streamer::{ChartsPayload, ConsumerStatusReporter, DeviceStreamPayload, PricesPayload, QueueName, ShutdownReceiver, TransactionsPayload, run_consumer};

use crate::consumers::runner::ChainConsumerRunner;
use crate::consumers::{consumer_config, reader_for_queue};
use crate::pusher::Pusher;

use device_stream_consumer::DeviceStreamConsumer;
use store_charts_consumer::StoreChartsConsumer;
use store_pending_transactions_consumer::StorePendingTransactionsConsumer;
use store_prices_consumer::StorePricesConsumer;

pub async fn run_consumer_store(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let settings = Arc::new(settings);

    futures::future::try_join_all(vec![
        tokio::spawn(run_store_transactions(settings.clone(), database.clone(), shutdown_rx.clone(), reporter.clone())),
        tokio::spawn(run_store_prices(settings.clone(), database.clone(), shutdown_rx.clone(), reporter.clone())),
        tokio::spawn(run_store_charts(settings.clone(), database.clone(), shutdown_rx.clone(), reporter.clone())),
        tokio::spawn(run_store_pending_transactions(settings.clone(), shutdown_rx.clone(), reporter.clone())),
        tokio::spawn(run_device_stream(settings.clone(), shutdown_rx.clone(), reporter.clone())),
    ])
    .await?;

    Ok(())
}

async fn run_store_transactions(
    settings: Arc<Settings>,
    database: Database,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new((*settings).clone(), database, QueueName::StoreTransactions, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::StoreTransactions;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let stream_producer = runner.stream_producer().await?;
            let config_cacher = ConfigCacher::new(runner.database.clone());
            let consumer = StoreTransactionsConsumer {
                database: runner.database.clone(),
                stream_producer,
                pusher: Pusher::new(runner.database.clone()),
                config: StoreTransactionsConsumerConfig {
                    swap_outdated_timeout: config_cacher.get_duration(ConfigKey::TransactionSwapOutdatedTimeout)?,
                    outdated_block_count: config_cacher.get_i64(ConfigKey::TransactionsOutdatedBlockCount)? as u64,
                    outdated_min_timeout: config_cacher.get_duration(ConfigKey::TransactionsOutdatedMinTimeout)?,
                    min_amount_usd: config_cacher.get_f64(ConfigKey::TransactionsMinAmountUsd)?,
                },
                vault_client: SwapVaultAddressClient::new(runner.cacher.clone()),
            };
            run_consumer::<TransactionsPayload, StoreTransactionsConsumer, usize>(
                &name,
                stream_reader,
                queue,
                Some(chain.as_ref()),
                consumer,
                runner.config,
                runner.shutdown_rx,
                runner.reporter,
            )
            .await
        })
        .await
}

async fn run_store_prices(
    settings: Arc<Settings>,
    database: Database,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::StorePrices;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown_rx).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let price_client = PriceClient::new(database.clone(), cacher_client);
    let config = ConfigCacher::new(database.clone());
    let ttl_seconds = config.get_duration(ConfigKey::PriceOutdated)?.as_secs() as i64;
    let consumer = StorePricesConsumer::new(database, price_client, ttl_seconds);
    run_consumer::<PricesPayload, StorePricesConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

async fn run_store_charts(
    settings: Arc<Settings>,
    database: Database,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::StoreCharts;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown_rx).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let price_client = PriceClient::new(database, cacher_client);
    let consumer = StoreChartsConsumer::new(price_client);
    run_consumer::<ChartsPayload, StoreChartsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

async fn run_device_stream(settings: Arc<Settings>, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::DeviceStreamEvents;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown_rx).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let consumer = DeviceStreamConsumer { cacher_client };
    run_consumer::<DeviceStreamPayload, DeviceStreamConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

async fn run_store_pending_transactions(
    settings: Arc<Settings>,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::StorePendingTransactions;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown_rx).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let consumer = StorePendingTransactionsConsumer::new(cacher);
    run_consumer::<TransactionId, StorePendingTransactionsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter)
        .await
}
