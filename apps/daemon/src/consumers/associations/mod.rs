pub mod fetch_coin_addresses_consumer;
pub mod fetch_nft_assets_addresses_consumer;
pub mod fetch_token_addresses_consumer;

use std::error::Error;
use std::sync::Arc;

use primitives::{Chain, NFTChain};
use settings::Settings;
use streamer::{ChainAddressPayload, ConsumerStatusReporter, QueueName, ShutdownReceiver, run_consumer};

use crate::consumers::chain_providers;
use crate::consumers::runner::ChainConsumerRunner;
use crate::consumers::transactions::FetchAddressTransactionsConsumer;

use fetch_coin_addresses_consumer::FetchCoinAddressesConsumer;
use fetch_nft_assets_addresses_consumer::FetchNftAssetsAddressesConsumer;
use fetch_token_addresses_consumer::FetchTokenAddressesConsumer;

pub async fn run_consumer_fetch_associations(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = Arc::new(settings);

    futures::future::try_join_all(vec![
        tokio::spawn(run_fetch_token_associations(settings.clone(), shutdown_rx.clone(), reporter.clone())),
        tokio::spawn(run_fetch_coin_associations(settings.clone(), shutdown_rx.clone(), reporter.clone())),
        tokio::spawn(run_fetch_nft_associations(settings.clone(), shutdown_rx.clone(), reporter.clone())),
        tokio::spawn(run_fetch_transaction_associations(settings.clone(), shutdown_rx.clone(), reporter.clone())),
    ])
    .await?;

    Ok(())
}

async fn run_fetch_token_associations(
    settings: Arc<Settings>,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new((*settings).clone(), QueueName::FetchTokenAssociations, shutdown_rx, reporter)
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
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new((*settings).clone(), QueueName::FetchCoinAssociations, shutdown_rx, reporter)
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
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chains: Vec<Chain> = NFTChain::all().into_iter().map(Into::into).collect();
    ChainConsumerRunner::new((*settings).clone(), QueueName::FetchNftAssociations, shutdown_rx, reporter)
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
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new((*settings).clone(), QueueName::FetchAddressTransactions, shutdown_rx, reporter)
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
