pub mod assets_consumer;
pub mod blocks_consumer;
pub mod transactions_consumer;
pub mod transactions_consumer_config;

use std::error::Error;
use std::sync::Arc;

pub use assets_consumer::FetchAssetsConsumer;
use gem_chain_rpc::{ChainBlockProvider, ChainProvider};
use primitives::Chain;
use settings::Settings;
use storage::{DatabaseClient, NodeStore};
use streamer::{ConsumerConfig, FetchAssetsPayload, FetchBlocksPayload, QueueName, StreamReader, TransactionsPayload};
use tokio::sync::Mutex;
pub use transactions_consumer::TransactionsConsumer;
pub use transactions_consumer_config::TransactionsConsumerConfig;

use crate::{consumers::blocks_consumer::FetchBlocksConsumer, parser_proxy::ParserProxy, Pusher};

pub async fn run_consumers(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::spawn(run_consumer_assets(settings.clone(), database.clone()));
    tokio::spawn(run_consumer_transactions(settings.clone(), database.clone()));
    std::future::pending::<()>().await;
    Ok(())
}

pub async fn run_consumer_assets(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let stream_reader = StreamReader::new(&settings.rabbitmq.url).await.unwrap();
    let nodes = {
        let mut db_guard = database.lock().await;
        db_guard.get_nodes()?.into_iter().map(|x| x.as_primitive()).collect::<Vec<_>>()
    };

    let providers = Chain::all()
        .into_iter()
        .map(|chain| Box::new(ParserProxy::new_from_nodes(&settings, chain, nodes.clone())) as Box<dyn ChainProvider>)
        .collect::<Vec<_>>();

    let consumer = FetchAssetsConsumer {
        providers,
        database: database.clone(),
    };
    streamer::run_consumer::<FetchAssetsPayload, FetchAssetsConsumer, usize>(
        "assets",
        stream_reader,
        QueueName::FetchAssets,
        consumer,
        ConsumerConfig::default(),
    )
    .await
}

pub async fn run_consumer_transactions(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let stream_reader = StreamReader::new(&settings.rabbitmq.url).await.unwrap();
    let stream_producer = streamer::StreamProducer::new(&settings.rabbitmq.url).await.unwrap();
    let pusher = Pusher::new(&settings.postgres.url);

    let consumer = TransactionsConsumer {
        database: database.clone(),
        stream_producer,
        pusher,
        config: TransactionsConsumerConfig::default(),
    };
    streamer::run_consumer::<TransactionsPayload, TransactionsConsumer, usize>(
        "transactions",
        stream_reader,
        QueueName::Transactions,
        consumer,
        ConsumerConfig::default(),
    )
    .await
}

pub async fn run_consumer_blocks(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let stream_reader = StreamReader::new(&settings.rabbitmq.url).await.unwrap();
    let stream_producer = streamer::StreamProducer::new(&settings.rabbitmq.url).await.unwrap();
    let nodes = database.lock().await.get_nodes()?.into_iter().map(|x| x.as_primitive()).collect::<Vec<_>>();

    let providers = Chain::all()
        .into_iter()
        .map(|chain| Box::new(ParserProxy::new_from_nodes(&settings, chain, nodes.clone())) as Box<dyn ChainBlockProvider>)
        .collect::<Vec<_>>();

    let consumer = FetchBlocksConsumer::new(providers, stream_producer);
    streamer::run_consumer::<FetchBlocksPayload, FetchBlocksConsumer, usize>(
        "blocks",
        stream_reader,
        QueueName::FetchBlocks,
        consumer,
        ConsumerConfig::default(),
    )
    .await
}
