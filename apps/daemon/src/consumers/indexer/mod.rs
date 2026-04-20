pub mod fetch_address_transactions_consumer;
pub mod fetch_assets_consumer;
pub mod fetch_blocks_consumer;
pub mod fetch_coin_addresses_consumer;
pub mod fetch_nft_assets_addresses_consumer;
pub mod fetch_token_addresses_consumer;

use std::error::Error;
use std::sync::Arc;

use cacher::CacherClient;
use primitives::{Chain, NFTChain};
use settings::Settings;
use storage::Database;
use streamer::{ChainAddressPayload, ConsumerStatusReporter, FetchAssetsPayload, FetchBlocksPayload, QueueName, ShutdownReceiver, StreamConnection, StreamReader, run_consumer};

use crate::consumers::runner::ChainConsumerRunner;
use crate::consumers::{chain_providers, consumer_config, reader_config};

use fetch_address_transactions_consumer::FetchAddressTransactionsConsumer;
use fetch_assets_consumer::FetchAssetsConsumer;
use fetch_blocks_consumer::FetchBlocksConsumer;
use fetch_coin_addresses_consumer::FetchCoinAddressesConsumer;
use fetch_nft_assets_addresses_consumer::FetchNftAssetsAddressesConsumer;
use fetch_token_addresses_consumer::FetchTokenAddressesConsumer;

pub async fn run_consumer_indexer(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
    only: Option<crate::model::IndexerConsumer>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    use crate::model::IndexerConsumer::*;

    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let settings = Arc::new(settings);

    let selected: Vec<crate::model::IndexerConsumer> = match only {
        Some(one) => vec![one],
        None => vec![
            FetchBlocks,
            FetchAssets,
            FetchTokenAssociations,
            FetchCoinAssociations,
            FetchNftAssociations,
            FetchAddressTransactions,
        ],
    };

    let handles: Vec<_> = selected
        .into_iter()
        .map(|kind| {
            let settings = settings.clone();
            let database = database.clone();
            let shutdown_rx = shutdown_rx.clone();
            let reporter = reporter.clone();
            tokio::spawn(async move {
                match kind {
                    FetchBlocks => run_fetch_blocks(settings, database, shutdown_rx, reporter).await,
                    FetchAssets => run_fetch_assets(settings, database, shutdown_rx, reporter).await,
                    FetchTokenAssociations => run_fetch_token_associations(settings, database, shutdown_rx, reporter).await,
                    FetchCoinAssociations => run_fetch_coin_associations(settings, database, shutdown_rx, reporter).await,
                    FetchNftAssociations => run_fetch_nft_associations(settings, database, shutdown_rx, reporter).await,
                    FetchAddressTransactions => run_fetch_transaction_associations(settings, database, shutdown_rx, reporter).await,
                }
            })
        })
        .collect();

    for handle in futures::future::join_all(handles).await {
        handle??;
    }
    Ok(())
}

async fn run_fetch_blocks(
    settings: Arc<Settings>,
    database: Database,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new((*settings).clone(), database, QueueName::FetchBlocks, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchBlocks;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let stream_producer = runner.stream_producer().await?;
            let consumer = FetchBlocksConsumer::new(chain_providers(&runner.settings, &name), stream_producer);
            run_consumer::<FetchBlocksPayload, FetchBlocksConsumer, usize>(
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

async fn run_fetch_assets(
    settings: Arc<Settings>,
    database: Database,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchAssets;
    let name = queue.to_string();
    let connection = StreamConnection::new(&settings.rabbitmq.url, name.clone()).await?;
    let config = reader_config(&settings.rabbitmq, name.clone());
    let stream_reader = StreamReader::from_connection(&connection, config).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let consumer = FetchAssetsConsumer {
        providers: chain_providers(&settings, &name),
        database,
        cacher,
    };
    run_consumer::<FetchAssetsPayload, FetchAssetsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

async fn run_fetch_token_associations(
    settings: Arc<Settings>,
    database: Database,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new((*settings).clone(), database, QueueName::FetchTokenAssociations, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchTokenAssociations;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let stream_producer = runner.stream_producer().await?;
            let consumer = FetchTokenAddressesConsumer::new(chain_providers(&runner.settings, &name), runner.database, stream_producer, runner.cacher);
            run_consumer::<ChainAddressPayload, FetchTokenAddressesConsumer, usize>(
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

async fn run_fetch_coin_associations(
    settings: Arc<Settings>,
    database: Database,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new((*settings).clone(), database, QueueName::FetchCoinAssociations, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchCoinAssociations;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let consumer = FetchCoinAddressesConsumer::new(chain_providers(&runner.settings, &name), runner.database, runner.cacher);
            run_consumer::<ChainAddressPayload, FetchCoinAddressesConsumer, String>(
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

async fn run_fetch_nft_associations(
    settings: Arc<Settings>,
    database: Database,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chains: Vec<Chain> = NFTChain::all().into_iter().map(Into::into).collect();
    ChainConsumerRunner::new((*settings).clone(), database, QueueName::FetchNftAssociations, shutdown_rx, reporter)
        .await?
        .run_for_chains(chains, |runner, chain| async move {
            FetchNftAssetsAddressesConsumer::run(
                runner.settings,
                runner.database,
                chain,
                &runner.connection,
                runner.cacher,
                runner.config,
                runner.shutdown_rx,
                runner.reporter,
            )
            .await
        })
        .await
}

async fn run_fetch_transaction_associations(
    settings: Arc<Settings>,
    database: Database,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new((*settings).clone(), database, QueueName::FetchAddressTransactions, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchAddressTransactions;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let stream_producer = runner.stream_producer().await?;
            let consumer = FetchAddressTransactionsConsumer::new(runner.database, chain_providers(&runner.settings, &name), stream_producer, runner.cacher);
            run_consumer::<ChainAddressPayload, FetchAddressTransactionsConsumer, usize>(
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
