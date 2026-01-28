use std::error::Error;

use api_connector::PusherClient;
use async_trait::async_trait;
use gem_tracing::info_with_fields;
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
        let success = &result.response.success;
        let logs = &result.response.logs;

        info_with_fields!("gorush response", counts = counts, success = success.as_str(), logs = format!("{:?}", logs));

        let failures = result.failures();

        if !failures.is_empty() {
            info_with_fields!(
                "push failures",
                count = failures.len(),
                failures = format!("{:?}", failures.iter().map(|f| (&f.notification.device_id, &f.error.error)).collect::<Vec<_>>())
            );
            self.stream_producer.publish_notifications_failed(NotificationsFailedPayload::new(failures)).await?;
        }

        Ok(counts)
    }
}
