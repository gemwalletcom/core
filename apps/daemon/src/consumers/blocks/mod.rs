pub mod fetch_blocks_consumer;

use std::error::Error;
use std::sync::Arc;

use settings::Settings;
use streamer::{ConsumerStatusReporter, FetchBlocksPayload, QueueName, ShutdownReceiver, run_consumer};

use crate::consumers::chain_providers;
use crate::consumers::runner::ChainConsumerRunner;

use fetch_blocks_consumer::FetchBlocksConsumer;

pub async fn run_consumer_fetch_blocks(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(settings, QueueName::FetchBlocks, shutdown_rx, reporter)
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
