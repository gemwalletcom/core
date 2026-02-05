pub mod assets_addresses_consumer;
pub mod fetch_assets_consumer;

pub use assets_addresses_consumer::AssetsAddressesConsumer;
pub use fetch_assets_consumer::FetchAssetsConsumer;

use std::error::Error;
use std::sync::Arc;

use cacher::CacherClient;
use settings::Settings;
use storage::Database;
use streamer::{AssetsAddressPayload, ConsumerStatusReporter, FetchAssetsPayload, QueueName, ShutdownReceiver, run_consumer};

use crate::consumers::{chain_providers, consumer_config, reader_for_queue};

pub async fn run_consumer_fetch_assets(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::FetchAssets;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let consumer = FetchAssetsConsumer {
        providers: chain_providers(&settings, &name),
        database,
        cacher,
    };
    run_consumer::<FetchAssetsPayload, FetchAssetsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

pub async fn run_consumer_store_assets_associations(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::StoreAssetsAssociations;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let consumer = AssetsAddressesConsumer::new(database);
    run_consumer::<AssetsAddressPayload, AssetsAddressesConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter)
        .await
}
