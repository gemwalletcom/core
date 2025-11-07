use std::error::Error;

use async_trait::async_trait;
use storage::{DatabaseClient, database::devices::DeviceFieldUpdate};
use streamer::{NotificationsFailedPayload, consumer::MessageConsumer};

pub struct NotificationsFailedConsumer {
    pub database: DatabaseClient,
}

impl NotificationsFailedConsumer {
    pub fn new(database: DatabaseClient) -> Self {
        Self { database }
    }
}

#[async_trait]
impl MessageConsumer<NotificationsFailedPayload, usize> for NotificationsFailedConsumer {
    async fn should_process(&mut self, _payload: NotificationsFailedPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&mut self, payload: NotificationsFailedPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let device_ids: Vec<String> = payload
            .failures
            .iter()
            .filter(|f| f.error.is_device_invalid())
            .filter_map(|f| f.notification.device_id.clone())
            .collect();

        if device_ids.is_empty() {
            return Ok(0);
        }

        Ok(self
            .database
            .devices()
            .update_device_fields(device_ids, vec![DeviceFieldUpdate::IsPushEnabled(false)])?)
    }
}
