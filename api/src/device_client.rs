extern crate rocket;
use std::error::Error;

use storage::{DatabaseClient, models::UpdateDevice};
use api_connector::PusherClient;
use api_connector::pusher::model::Notification;

pub struct DevicesClient {
    database: DatabaseClient,
    pusher: PusherClient,
    pusher_topic: String,
}

impl DevicesClient {
    pub async fn new(
        database_url: &str,
        pusher: PusherClient,
        pusher_topic: String,
    ) -> Self {
        let database = DatabaseClient::new(database_url);
        Self {
            database,
            pusher,
            pusher_topic,
        }
    }

    pub fn add_device(&mut self, device: primitives::device::Device) -> Result<primitives::device::Device, Box<dyn Error>> {
        let add_device = UpdateDevice::from_primitive(device.clone());
        let device = self.database.add_device(add_device)?;
        Ok(device.as_primitive())
    }

    pub fn get_device(&mut self, device_id: &str) -> Result<primitives::Device, Box<dyn Error>> {
        let device = self.database.get_device(device_id)?;
        Ok(device.as_primitive())
    }

    pub fn update_device(&mut self, device: primitives::device::Device) -> Result<primitives::device::Device, Box<dyn Error>> {
        let update_device = UpdateDevice::from_primitive(device);
        let device = self.database.update_device(update_device)?;
        Ok(device.as_primitive())
    }

    pub async fn send_push_notification_device(&mut self, device_id: &str) -> Result<bool, Box<dyn Error>> {
        let device = self.get_device(device_id)?;
        let device_token = self.database.get_device_token(device_id)?;
        let notification = Notification { 
            tokens: vec![device_token], 
            platform: device.platform.as_i32(), 
            title: "Test Notification".to_string(), 
            message: "Test Message".to_string(), 
            topic: self.pusher_topic.to_string(),
        };
        let result = self.pusher.push(notification).await?;
        Ok(result.counts > 0)
    }

    pub fn delete_device(&mut self, device_id: &str) -> Result<usize, Box<dyn Error>> {
        Ok(self.database.delete_device(device_id)?)
    }
}