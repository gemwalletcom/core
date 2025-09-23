use std::{
    error::Error,
    fmt::Display,
    time::{Duration, Instant},
};

use crate::{QueueName, StreamReader};
use async_trait::async_trait;
use gem_tracing::{error_with_fields, info_with_fields};
use serde::Deserialize;
use tokio;

pub struct ConsumerConfig {
    pub timeout_on_error: Duration,
    pub skip_on_error: bool,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            timeout_on_error: Duration::from_secs(1),
            skip_on_error: true,
        }
    }
}

#[async_trait]
pub trait MessageConsumer<P, R> {
    async fn process(&mut self, payload: P) -> Result<R, Box<dyn Error + Send + Sync>>;
    async fn should_process(&mut self, payload: P) -> Result<bool, Box<dyn Error + Send + Sync>>;
}

pub async fn run_consumer<P, C, R>(
    name: &str,
    mut stream_reader: StreamReader,
    queue_name: QueueName,
    mut consumer: C,
    config: ConsumerConfig,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    P: Clone + Send + Display + 'static,
    C: MessageConsumer<P, R> + Send + 'static,
    R: std::default::Default + std::fmt::Debug,
    for<'a> P: Deserialize<'a> + std::fmt::Debug,
{
    info_with_fields!("running consumer", consumer = name, queue = queue_name.to_string());

    stream_reader
        .read::<P, _>(queue_name, move |payload| {
            info_with_fields!("consumer received", consumer = name, payload = payload.to_string());
            let start = Instant::now();
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    match consumer.should_process(payload.clone()).await {
                        Ok(true) => consumer.process(payload.clone()).await,
                        Ok(false) => {
                            info_with_fields!("consumer should not process", consumer = name, payload = payload.to_string());
                            Ok(R::default())
                        }
                        Err(e) => Err(e),
                    }
                })
            });
            match result {
                Ok(result) => {
                    info_with_fields!(
                        "consumer result",
                        consumer = name,
                        result = format!("{:?}", result),
                        elapsed_ms = start.elapsed().as_millis().to_string()
                    );
                    Ok(())
                }
                Err(e) => {
                    error_with_fields!(
                        "consumer error",
                        &*e,
                        consumer = name,
                        payload = payload.to_string(),
                        elapsed_ms = start.elapsed().as_millis().to_string()
                    );
                    std::thread::sleep(config.timeout_on_error);
                    if config.skip_on_error { Ok(()) } else { Err(e) }
                }
            }
        })
        .await
}
