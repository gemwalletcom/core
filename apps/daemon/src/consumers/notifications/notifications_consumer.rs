use std::error::Error;

use api_connector::PusherClient;
use async_trait::async_trait;
use streamer::{NotificationsFailedPayload, NotificationsPayload, StreamProducer, StreamProducerQueue, consumer::MessageConsumer};

pub struct NotificationsConsumer {
    pub pusher: PusherClient,
    pub stream_producer: StreamProducer,
}

impl NotificationsConsumer {
    pub fn new(pusher: PusherClient, stream_producer: StreamProducer) -> Self {
        Self { pusher, stream_producer }
    }
}

#[async_trait]
impl MessageConsumer<NotificationsPayload, usize> for NotificationsConsumer {
    async fn should_process(&self, _payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: NotificationsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let result = self.pusher.push_notifications(payload.notifications).await?;
        let counts = result.response.counts as usize;
        let failures = result.failures();

        if !failures.is_empty() {
            self.stream_producer
                .publish_notifications_failed(NotificationsFailedPayload::new(failures))
                .await?;
        }

        Ok(counts)
    }
}
