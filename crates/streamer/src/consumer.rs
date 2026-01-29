use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::{
    error::Error,
    fmt::Display,
    time::{Duration, Instant},
};

use crate::{QueueName, ShutdownReceiver, StreamReader};
use async_trait::async_trait;
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use serde::Deserialize;
use tokio;

#[derive(Clone)]
pub struct ConsumerConfig {
    pub timeout_on_error: Duration,
    pub skip_on_error: bool,
    pub delay: Duration,
}

enum ProcessResult<R> {
    Processed(R),
    Skipped,
    Error(Box<dyn Error + Send + Sync>),
}

pub trait ConsumerStatusReporter: Send + Sync {
    fn report_success(&self, name: &str, duration: u64, result: &str) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
    fn report_error(&self, name: &str, error: &str) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
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
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    P: Clone + Send + Display + 'static,
    C: MessageConsumer<P, R> + Send + 'static,
    R: std::fmt::Debug,
    for<'a> P: Deserialize<'a> + std::fmt::Debug,
{
    info_with_fields!("running consumer", consumer = name, queue = queue_name.to_string(), routing_key = routing_key.unwrap_or(""));
    stream_reader
        .read::<P, _>(
            queue_name,
            routing_key,
            |payload| process_message(name, &consumer, &config, &reporter, payload),
            shutdown_rx,
        )
        .await
}

fn process_message<P, C, R>(name: &str, consumer: &C, config: &ConsumerConfig, reporter: &Arc<dyn ConsumerStatusReporter>, payload: P) -> Result<(), Box<dyn Error + Send + Sync>>
where
    P: Clone + Send + Display + 'static,
    C: MessageConsumer<P, R> + Send + 'static,
    R: std::fmt::Debug,
    for<'a> P: Deserialize<'a> + std::fmt::Debug,
{
    info_with_fields!("processing", consumer = name, payload = payload.to_string());
    let start = Instant::now();
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match consumer.should_process(payload.clone()).await {
                Ok(true) => match consumer.process(payload.clone()).await {
                    Ok(r) => ProcessResult::Processed(r),
                    Err(e) => ProcessResult::Error(e),
                },
                Ok(false) => ProcessResult::Skipped,
                Err(e) => ProcessResult::Error(e),
            }
        })
    });
    match result {
        ProcessResult::Processed(value) => {
            let duration = start.elapsed().as_millis() as u64;
            let result_str = format!("{:?}", value);
            info_with_fields!(
                "processed",
                consumer = name,
                payload = payload.to_string(),
                result = result_str,
                elapsed = DurationMs(start.elapsed())
            );
            tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(reporter.report_success(name, duration, &result_str)));
            if !config.delay.is_zero() {
                tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(tokio::time::sleep(config.delay)));
            }
            Ok(())
        }
        ProcessResult::Skipped => {
            info_with_fields!("skipped", consumer = name, payload = payload.to_string(), elapsed = DurationMs(start.elapsed()));
            Ok(())
        }
        ProcessResult::Error(e) => {
            error_with_fields!("error", &*e, consumer = name, payload = payload.to_string(), elapsed = DurationMs(start.elapsed()));
            let error_msg = format!("{}", e);
            tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(reporter.report_error(name, &error_msg)));
            if !config.timeout_on_error.is_zero() {
                tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(tokio::time::sleep(config.timeout_on_error)));
            }
            if config.skip_on_error { Ok(()) } else { Err(e) }
        }
    }
}
