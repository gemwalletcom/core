pub mod assets_consumer;
pub mod blocks_consumer;
pub mod transactions_consumer;
pub mod transactions_consumer_config;

use std::error::Error;

pub use assets_consumer::FetchAssetsConsumer;
use gem_chain_rpc::{ChainBlockProvider, ChainProvider};
use primitives::Chain;
use settings::Settings;
use storage::DatabaseClient;
use streamer::{ConsumerConfig, FetchAssetsPayload, FetchBlocksPayload, QueueName, StreamReader, TransactionsPayload};
pub use transactions_consumer::TransactionsConsumer;
pub use transactions_consumer_config::TransactionsConsumerConfig;

use crate::{consumers::blocks_consumer::FetchBlocksConsumer, parser_proxy::ParserProxy, Pusher};

pub async fn run_consumers(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::spawn(run_consumer_assets(settings.clone()));
    tokio::spawn(run_consumer_transactions(settings.clone()));
    std::future::pending::<()>().await;
    Ok(())
}

pub async fn run_consumer_assets(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    let stream_reader = StreamReader::new(&settings.rabbitmq.url).await.unwrap();
    let mut database = storage::DatabaseClient::new(&settings.postgres.url);
    let nodes = database.get_nodes()?.into_iter().map(|x| x.as_primitive()).collect::<Vec<_>>();

    let providers = Chain::all()
        .into_iter()
        .map(|chain| Box::new(ParserProxy::new_from_nodes(&settings, chain, nodes.clone())) as Box<dyn ChainProvider>)
        .collect::<Vec<_>>();

    let consumer = FetchAssetsConsumer { providers, database };
    streamer::run_consumer::<FetchAssetsPayload, FetchAssetsConsumer, usize>(
        "assets",
        stream_reader,
        QueueName::FetchAssets,
        consumer,
        ConsumerConfig::default(),
    )
    .await
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
    streamer::run_consumer::<TransactionsPayload, TransactionsConsumer, usize>(
        "transactions",
        stream_reader,
        QueueName::Transactions,
        consumer,
        ConsumerConfig::default(),
    )
    .await
}

pub async fn run_consumer_blocks(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    let stream_reader = StreamReader::new(&settings.rabbitmq.url).await.unwrap();
    let stream_producer = streamer::StreamProducer::new(&settings.rabbitmq.url).await.unwrap();
    let mut database = DatabaseClient::new(&settings.postgres.url);
    let nodes = database.get_nodes()?.into_iter().map(|x| x.as_primitive()).collect::<Vec<_>>();

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
