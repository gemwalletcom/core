use std::{error::Error, time::Instant};

use api_connector::PusherClient;
use streamer::{NotificationsPayload, QueueName, StreamReader};

pub struct NotificationsConsumer {
    pusher: PusherClient,
    stream_reader: StreamReader,
}

impl NotificationsConsumer {
    pub fn new(pusher_client: PusherClient, stream_reader: StreamReader) -> Self {
        Self {
            pusher: pusher_client,
            stream_reader,
        }
    }
    pub async fn run(&mut self, service: &str, queue_name: QueueName) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_reader
            .read::<NotificationsPayload, _>(queue_name.clone(), |payload| {
                let count = payload.notifications.len();
                println!("{} received message: queue: {}, count: {}", service, queue_name.clone(), count);

                let start = Instant::now();
                let result = tokio::task::block_in_place(|| {
                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(async { self.pusher.push_notifications(payload.notifications).await })
                });
                let elapsed = start.elapsed();

                match &result {
                    Ok(_) => {
                        println!("{} processed: queue: {}, count: {}, elapsed: {:?}", service, queue_name.clone(), count, elapsed);
                    }
                    Err(error) => {
                        println!(
                            "{} failed: queue: {}, count: {}, elapsed: {:?}, error: {}",
                            service,
                            queue_name.clone(),
                            count,
                            elapsed,
                            error
                        );
                    }
                }

                result.map(|_| ()).map_err(Into::into)
            })
            .await
    }
}
