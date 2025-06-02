pub mod assets_consumer;
pub mod transactions_consumer;
pub mod transactions_consumer_config;

use std::error::Error;

pub use assets_consumer::FetchAssetsConsumer;
use settings::Settings;
use storage::DatabaseClient;
use streamer::{FetchAssetsPayload, QueueName, StreamReader, TransactionsPayload};
pub use transactions_consumer::TransactionsConsumer;
pub use transactions_consumer_config::TransactionsConsumerConfig;

use crate::Pusher;

pub async fn run_consumers(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::spawn(run_consumer_assets(settings.clone()));
    tokio::spawn(run_consumer_transactions(settings.clone()));
    std::future::pending::<()>().await;
    Ok(())
}

pub async fn run_consumer_assets(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    let stream_reader = StreamReader::new(&settings.rabbitmq.url).await.unwrap();
    let database = storage::DatabaseClient::new(&settings.postgres.url);
    let consumer = FetchAssetsConsumer { database };
    streamer::run_consumer::<FetchAssetsPayload, FetchAssetsConsumer, usize>("assets", stream_reader, QueueName::FetchAssets, consumer).await
}

pub async fn run_consumer_transactions(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    let stream_reader = StreamReader::new(&settings.rabbitmq.url).await.unwrap();
    let database = DatabaseClient::new(&settings.postgres.url);
    let stream_producer = streamer::StreamProducer::new(&settings.rabbitmq.url).await.unwrap();
    let pusher = Pusher::new(&settings.postgres.url);

    let consumer = TransactionsConsumer {
        database,
        stream_producer,
        pusher,
        config: TransactionsConsumerConfig::default(),
    };
    streamer::run_consumer::<TransactionsPayload, TransactionsConsumer, usize>("transactions", stream_reader, QueueName::Transactions, consumer).await
}
