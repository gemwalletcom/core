pub mod connection;
pub mod consumer;
pub mod exchange;
pub mod payload;
pub mod queue;
pub mod steam_producer_queue;
pub mod stream_producer;
pub mod stream_reader;

use std::error::Error;
use std::future::Future;
use std::time::Duration;

use gem_tracing::info_with_fields;

#[derive(Clone)]
pub struct Retry {
    pub delay: Duration,
    pub timeout: Duration,
}

impl Retry {
    pub fn new(delay: Duration, timeout: Duration) -> Self {
        Self { delay, timeout }
    }
}

pub async fn with_retry<F, Fut, T>(retry: &Retry, name: &str, mut f: F) -> Result<T, Box<dyn Error + Send + Sync>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
{
    let mut delay = retry.delay;
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        match f().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                info_with_fields!(
                    "rabbitmq connect retry",
                    connection = name,
                    attempt = attempt,
                    delay_secs = delay.as_secs(),
                    error = err.to_string()
                );
                tokio::time::sleep(delay).await;
                delay = (delay * 2).min(retry.timeout);
            }
        }
    }
}

pub use connection::StreamConnection;
pub use consumer::ConsumerConfig;
pub use consumer::ConsumerStatusReporter;
pub use consumer::run_consumer;
pub use exchange::ExchangeName;
pub use lapin::ExchangeKind;
pub use payload::*;
pub use primitives::{AssetId, PushErrorLog};
pub use queue::QueueName;
pub use steam_producer_queue::StreamProducerQueue;
pub use stream_producer::{StreamProducer, StreamProducerConfig};
pub use stream_reader::{ShutdownReceiver, StreamReader, StreamReaderConfig};
