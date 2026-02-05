pub mod fetch_address_transactions_consumer;
pub mod store_transactions_consumer;
pub mod store_transactions_consumer_config;

pub use fetch_address_transactions_consumer::FetchAddressTransactionsConsumer;
pub use store_transactions_consumer::StoreTransactionsConsumer;
pub use store_transactions_consumer_config::StoreTransactionsConsumerConfig;

use std::error::Error;
use std::sync::Arc;

use settings::Settings;
use storage::{ConfigCacher, Database};
use streamer::{ConsumerStatusReporter, QueueName, ShutdownReceiver, TransactionsPayload, run_consumer};

use crate::consumers::runner::ChainConsumerRunner;
use crate::pusher::Pusher;

pub async fn run_consumer_store_transactions(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(settings, QueueName::StoreTransactions, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::StoreTransactions;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let stream_producer = runner.stream_producer().await?;
            let database = Database::new(&runner.settings.postgres.url, runner.settings.postgres.pool);
            let consumer = StoreTransactionsConsumer {
                database: database.clone(),
                config_cacher: ConfigCacher::new(database.clone()),
                stream_producer,
                pusher: Pusher::new(database.clone()),
                config: StoreTransactionsConsumerConfig {},
            };
            run_consumer::<TransactionsPayload, StoreTransactionsConsumer, usize>(
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
