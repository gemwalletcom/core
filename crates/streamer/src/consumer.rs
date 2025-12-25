use std::{
    error::Error,
    fmt::Display,
    time::{Duration, Instant},
};

use crate::{QueueName, StreamReader};
use async_trait::async_trait;
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use serde::Deserialize;
use tokio;

#[derive(Clone)]
pub struct ConsumerConfig {
    pub timeout_on_error: Duration,
    pub skip_on_error: bool,
}

#[async_trait]
pub trait MessageConsumer<P, R> {
    async fn process(&self, payload: P) -> Result<R, Box<dyn Error + Send + Sync>>;
    async fn should_process(&self, payload: P) -> Result<bool, Box<dyn Error + Send + Sync>>;
}

pub async fn run_consumer<P, C, R>(
    name: &str,
    mut stream_reader: StreamReader,
    queue_name: QueueName,
    routing_key: Option<&str>,
    consumer: C,
    config: ConsumerConfig,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    P: Clone + Send + Display + 'static,
    C: MessageConsumer<P, R> + Send + 'static,
    R: std::default::Default + std::fmt::Debug,
    for<'a> P: Deserialize<'a> + std::fmt::Debug,
{
    info_with_fields!(
        "running consumer",
        consumer = name,
        queue = queue_name.to_string(),
        routing_key = routing_key.unwrap_or("")
    );
    stream_reader
        .read::<P, _>(queue_name, routing_key, |payload| process_message(name, &consumer, &config, payload))
        .await
}

fn process_message<P, C, R>(name: &str, consumer: &C, config: &ConsumerConfig, payload: P) -> Result<(), Box<dyn Error + Send + Sync>>
where
    P: Clone + Send + Display + 'static,
    C: MessageConsumer<P, R> + Send + 'static,
    R: std::default::Default + std::fmt::Debug,
    for<'a> P: Deserialize<'a> + std::fmt::Debug,
{
    info_with_fields!("processing", consumer = name, payload = payload.to_string());
    let start = Instant::now();
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match consumer.should_process(payload.clone()).await {
                Ok(true) => consumer.process(payload.clone()).await,
                Ok(false) => {
                    info_with_fields!("skipped", consumer = name, payload = payload.to_string());
                    Ok(R::default())
                }
                Err(e) => Err(e),
            }
        })
    });
    match result {
        Ok(result) => {
            info_with_fields!(
                "processed",
                consumer = name,
                payload = payload.to_string(),
                result = format!("{:?}", result),
                elapsed = DurationMs(start.elapsed())
            );
            Ok(())
        }
        Err(e) => {
            error_with_fields!(
                "error",
                &*e,
                consumer = name,
                payload = payload.to_string(),
                elapsed = DurationMs(start.elapsed())
            );
            if !config.timeout_on_error.is_zero() {
                tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(tokio::time::sleep(config.timeout_on_error)));
            }
            if config.skip_on_error { Ok(()) } else { Err(e) }
        }
    }
}
