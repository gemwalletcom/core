mod nft_collection_asset_consumer;
mod nft_collection_consumer;
use nft_collection_asset_consumer::UpdateNftCollectionAssetsConsumer;
use nft_collection_consumer::UpdateNftCollectionConsumer;

use job_runner::{JobStatusReporter, ShutdownReceiver, run_job};
use settings::Settings;
use std::sync::Arc;
use std::time::Duration;
use streamer::{ConsumerConfig, ConsumerStatusReporter, FetchNFTCollectionAssetPayload, FetchNFTCollectionPayload, QueueName, StreamReader, StreamReaderConfig, run_consumer};
use tokio::task::JoinHandle;

fn consumer_config(consumer: &settings::Consumer) -> ConsumerConfig {
    ConsumerConfig {
        timeout_on_error: consumer.error.timeout,
        skip_on_error: consumer.error.skip,
        delay: consumer.delay,
    }
}

pub async fn jobs(
    settings: Settings,
    reporter: Arc<dyn JobStatusReporter>,
    consumer_reporter: Arc<dyn ConsumerStatusReporter>,
    shutdown_rx: ShutdownReceiver,
) -> Vec<JoinHandle<()>> {
    let settings_arc = Arc::new(settings);

    let settings = settings_arc.clone();
    let shutdown_rx_clone = shutdown_rx.clone();
    let consumer_reporter_clone = consumer_reporter.clone();
    let nft_collection_consumer_job = tokio::spawn(run_job(
        "consume_nft_collections",
        Duration::from_secs(u64::MAX),
        reporter.clone(),
        shutdown_rx.clone(),
        move || {
            let settings = settings.clone();
            let shutdown_rx = shutdown_rx_clone.clone();
            let reporter = consumer_reporter_clone.clone();
            async move {
                let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), "update_nft_collection".to_string(), settings.rabbitmq.prefetch);
                let stream_reader = StreamReader::new(config).await.unwrap();
                let consumer = UpdateNftCollectionConsumer::new();

                run_consumer::<FetchNFTCollectionPayload, UpdateNftCollectionConsumer, usize>(
                    "consume_nft_collections",
                    stream_reader,
                    QueueName::FetchNFTCollection,
                    None,
                    consumer,
                    consumer_config(&settings.consumer),
                    shutdown_rx,
                    reporter,
                )
                .await
            }
        },
    ));

    let settings = settings_arc.clone();
    let shutdown_rx_clone = shutdown_rx.clone();
    let nft_collection_assets_consumer_job = tokio::spawn(run_job(
        "consume_nft_collection_assets",
        Duration::from_secs(u64::MAX),
        reporter.clone(),
        shutdown_rx,
        move || {
            let settings = settings.clone();
            let shutdown_rx = shutdown_rx_clone.clone();
            let reporter = consumer_reporter.clone();
            async move {
                let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), "nft_collection_assets".to_string(), settings.rabbitmq.prefetch);
                let stream_reader = StreamReader::new(config).await.unwrap();
                let consumer = UpdateNftCollectionAssetsConsumer::new();

                run_consumer::<FetchNFTCollectionAssetPayload, UpdateNftCollectionAssetsConsumer, usize>(
                    "consume_nft_collection_assets",
                    stream_reader,
                    QueueName::FetchNFTCollectionAssets,
                    None,
                    consumer,
                    consumer_config(&settings.consumer),
                    shutdown_rx,
                    reporter,
                )
                .await
            }
        },
    ));

    vec![nft_collection_consumer_job, nft_collection_assets_consumer_job]
}
