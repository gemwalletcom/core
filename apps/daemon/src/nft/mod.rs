mod nft_collection_asset_consumer;
mod nft_collection_consumer;
use nft_collection_asset_consumer::UpdateNftCollectionAssetsConsumer;
use nft_collection_consumer::UpdateNftCollectionConsumer;

use job_runner::run_job;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use streamer::{ConsumerConfig, FetchNFTCollectionAssetPayload, FetchNFTCollectionPayload, QueueName, StreamReader, run_consumer};

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let settings_arc = Arc::new(settings);

    let settings = settings_arc.clone();
    let nft_collection_consumer_job = run_job("update nft collection consumer", Duration::from_secs(u64::MAX), move || {
        let settings = settings.clone();
        async move {
            let stream_reader = StreamReader::new(&settings.rabbitmq.url, "update_nft_collection").await.unwrap();
            let consumer = UpdateNftCollectionConsumer::new();

            run_consumer::<FetchNFTCollectionPayload, UpdateNftCollectionConsumer, usize>(
                "update nft collection consumer",
                stream_reader,
                QueueName::FetchNFTCollection,
                consumer,
                ConsumerConfig::default(),
            )
            .await
        }
    });

    let settings = settings_arc.clone();
    let nft_collection_assets_consumer_job = run_job("update nft collection assets consumer", Duration::from_secs(u64::MAX), move || {
        let settings = settings.clone();
        async move {
            let stream_reader = StreamReader::new(&settings.rabbitmq.url, "nft_collection_assets").await.unwrap();
            let consumer = UpdateNftCollectionAssetsConsumer::new();

            run_consumer::<FetchNFTCollectionAssetPayload, UpdateNftCollectionAssetsConsumer, usize>(
                "update nft collection assets consumer",
                stream_reader,
                QueueName::FetchNFTCollectionAssets,
                consumer,
                ConsumerConfig::default(),
            )
            .await
        }
    });

    vec![Box::pin(nft_collection_consumer_job), Box::pin(nft_collection_assets_consumer_job)]
}
