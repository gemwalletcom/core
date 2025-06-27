use std::error::Error;

use api_connector::PusherClient;
use async_trait::async_trait;
use streamer::{consumer::MessageConsumer, NotificationsPayload};

pub struct NotificationsConsumer {
    pub _pusher: PusherClient,
}

impl NotificationsConsumer {
    pub fn new(_pusher: PusherClient) -> Self {
        Self { _pusher }
    }
}

#[async_trait]
impl MessageConsumer<NotificationsPayload, usize> for NotificationsConsumer {
    async fn should_process(&mut self, _payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }
    async fn process(&mut self, _payload: NotificationsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        return Ok(0);
        // let count = payload.notifications.len();
        // if count == 0 {
        //     return Ok(0);
        // }
        // Ok(self.pusher.push_notifications(payload.notifications).await?.counts as usize)
    }
}
