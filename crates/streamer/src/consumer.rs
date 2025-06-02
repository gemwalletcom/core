use std::{error::Error, time::Instant};

use crate::{QueueName, StreamReader};
use async_trait::async_trait;
use serde::Deserialize;
use tokio;

#[async_trait]
pub trait MessageConsumer<P, R> {
    async fn process(&mut self, payload: P) -> Result<R, Box<dyn Error + Send + Sync>>;
}

pub async fn run_consumer<P, C, R>(
    name: &str,
    mut stream_reader: StreamReader,
    queue_name: QueueName,
    mut consumer: C,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    P: Clone + Send + 'static,
    C: MessageConsumer<P, R> + Send + 'static,
    R: std::fmt::Debug,
    for<'a> P: Deserialize<'a> + std::fmt::Debug,
{
    println!("Running consumer {} for queue {}", name, queue_name);

    // Process messages from the stream
    stream_reader
        .read::<P, _>(queue_name, move |payload| {
            let start = Instant::now();
            let result = tokio::task::block_in_place(|| {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async { consumer.process(payload.clone()).await })
            });
            match result {
                Ok(result) => {
                    println!("consumer {} processed message result: {:?}, elapsed: {:?}", name, result, start.elapsed());
                    Ok(())
                }
                Err(e) => {
                    println!("consumer {} Error processing message: {}, elapsed: {:?}", name, e, start.elapsed());
                    Err(e)
                }
            }
        })
        .await
}
