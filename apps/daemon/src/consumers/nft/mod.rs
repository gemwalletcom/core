mod nft_collection_asset_consumer;
mod nft_collection_consumer;

use nft_collection_asset_consumer::UpdateNftCollectionAssetsConsumer;
use nft_collection_consumer::UpdateNftCollectionConsumer;

use super::consumer_config;
use futures::future::try_join_all;
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use streamer::{ConsumerStatusReporter, FetchNFTCollectionAssetPayload, FetchNFTCollectionPayload, QueueName, ShutdownReceiver, StreamReader, StreamReaderConfig, run_consumer};

pub async fn run_consumer_nft_collections(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = Arc::new(settings);

    try_join_all(vec![
        tokio::spawn(run_collections_consumer(settings.clone(), shutdown_rx.clone(), reporter.clone())),
        tokio::spawn(run_collection_assets_consumer(settings, shutdown_rx, reporter)),
    ])
    .await?;

    Ok(())
}

async fn run_collections_consumer(settings: Arc<Settings>, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchNFTCollection;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let consumer = UpdateNftCollectionConsumer::new();

    run_consumer::<FetchNFTCollectionPayload, UpdateNftCollectionConsumer, usize>(
        "consume_nft_collections",
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

async fn run_collection_assets_consumer(
    settings: Arc<Settings>,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchNFTCollectionAssets;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let consumer = UpdateNftCollectionAssetsConsumer::new();

    run_consumer::<FetchNFTCollectionAssetPayload, UpdateNftCollectionAssetsConsumer, usize>(
        "consume_nft_collection_assets",
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
