use std::error::Error;

use api_connector::PusherClient;
use async_trait::async_trait;
use streamer::{NotificationsPayload, consumer::MessageConsumer};

pub struct NotificationsConsumer {
    pub pusher: PusherClient,
}

impl NotificationsConsumer {
    pub fn new(pusher: PusherClient) -> Self {
        Self { pusher }
    }
}

#[async_trait]
impl MessageConsumer<NotificationsPayload, usize> for NotificationsConsumer {
    async fn should_process(&mut self, _payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }
    async fn process(&mut self, payload: NotificationsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.pusher.push_notifications(payload.notifications).await?.counts as usize)
    }
}
