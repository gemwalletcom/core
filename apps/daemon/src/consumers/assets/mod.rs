mod update_coin_info_consumer;

use std::error::Error;
use std::sync::Arc;

use cacher::CacherClient;
use coingecko::CoinGeckoClient;
use settings::Settings;
use storage::Database;
use streamer::{ConsumerStatusReporter, QueueName, UpdateCoinInfoPayload};

use crate::consumers::{consumer_config, reader_for_queue};
use crate::shutdown::ShutdownReceiver;
use crate::worker::assets::asset_updater::AssetProcessor;

use update_coin_info_consumer::UpdateCoinInfoConsumer;

pub async fn run_consumer_assets(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::UpdateCoinInfo;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);
    let processor = AssetProcessor::new(coingecko_client, database, cacher_client);
    let consumer = UpdateCoinInfoConsumer::new(processor);

    streamer::run_consumer::<UpdateCoinInfoPayload, UpdateCoinInfoConsumer, usize>(
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
