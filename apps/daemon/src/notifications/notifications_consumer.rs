use std::error::Error;

use api_connector::PusherClient;
use async_trait::async_trait;
use streamer::{consumer::MessageConsumer, NotificationsPayload};

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
    async fn process(&mut self, payload: NotificationsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let count = payload.notifications.len();
        if count == 0 {
            return Ok(0);
        }
        Ok(self.pusher.push_notifications(payload.notifications).await?.counts as usize)
    }
}
