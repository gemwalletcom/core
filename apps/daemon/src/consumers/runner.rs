use std::error::Error;
use std::sync::Arc;

use cacher::CacherClient;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::Chain;
use settings::Settings;
use storage::Database;
use streamer::{ConsumerConfig, ConsumerStatusReporter, ShutdownReceiver, StreamConnection, StreamProducer, StreamReader};

use crate::consumers::{consumer_config, reader_config};

#[derive(Clone)]
pub struct ChainConsumerRunner {
    pub settings: Settings,
    pub database: Database,
    pub connection: StreamConnection,
    pub cacher: CacherClient,
    pub config: ConsumerConfig,
    pub shutdown_rx: ShutdownReceiver,
    pub reporter: Arc<dyn ConsumerStatusReporter>,
    queue: streamer::QueueName,
}

impl ChainConsumerRunner {
    pub async fn new(
        settings: Settings,
        queue: streamer::QueueName,
        shutdown_rx: ShutdownReceiver,
        reporter: Arc<dyn ConsumerStatusReporter>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let database = Database::new(&settings.postgres.url, settings.postgres.pool);
        let connection = StreamConnection::new(&settings.rabbitmq.url, queue.to_string()).await?;
        let cacher = CacherClient::new(&settings.redis.url).await;
        let config = consumer_config(&settings.consumer);
        Ok(Self {
            settings,
            database,
            connection,
            cacher,
            config,
            shutdown_rx,
            reporter,
            queue,
        })
    }

    pub async fn stream_reader(&self) -> Result<StreamReader, Box<dyn Error + Send + Sync>> {
        let config = reader_config(&self.settings.rabbitmq, self.connection.name().to_string());
        StreamReader::from_connection(&self.connection, config).await
    }

    pub async fn stream_producer(&self) -> Result<StreamProducer, Box<dyn Error + Send + Sync>> {
        StreamProducer::from_connection(&self.connection).await
    }

    pub async fn run<F, Fut>(self, f: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(Self, Chain) -> Fut + Clone + Send + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send + 'static,
    {
        self.run_for_chains(Chain::all(), f).await
    }

    pub async fn run_for_chains<F, Fut>(self, chains: Vec<Chain>, f: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(Self, Chain) -> Fut + Clone + Send + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send + 'static,
    {
        info_with_fields!("running consumer", consumer = self.queue.to_string(), chains = chains.len());
        let tasks = chains.into_iter().map(|chain| {
            let runner = self.clone();
            let f = f.clone();
            async move { (chain, f(runner, chain).await) }
        });

        for (chain, result) in futures::future::join_all(tasks).await {
            if let Err(err) = result {
                error_with_fields!("consumer chain error", &*err, chain = chain.as_ref());
                self.reporter.report_error(&self.queue.to_string(), &format!("{:?}", err)).await;
            }
        }
        Ok(())
    }
}
