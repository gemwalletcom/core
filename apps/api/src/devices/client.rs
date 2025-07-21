use api_connector::PusherClient;
use primitives::{GorushNotification, PushNotification, PushNotificationTypes};
use std::error::Error;
use storage::{models::UpdateDevice, DatabaseClient};

pub struct DevicesClient {
    database: DatabaseClient,
    pusher: PusherClient,
}

impl DevicesClient {
    pub async fn new(database_url: &str, pusher: PusherClient) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database, pusher }
    }

    pub fn add_device(&mut self, device: primitives::device::Device) -> Result<primitives::device::Device, Box<dyn Error + Send + Sync>> {
        let add_device = UpdateDevice::from_primitive(device.clone());
        let device = self.database.devices().add_device(add_device)?;
        Ok(device)
    }

    pub fn get_device(&mut self, device_id: &str) -> Result<primitives::Device, Box<dyn Error + Send + Sync>> {
        let device = self.database.devices().get_device(device_id)?;
        Ok(device)
    }

    pub fn update_device(&mut self, device: primitives::device::Device) -> Result<primitives::device::Device, Box<dyn Error + Send + Sync>> {
        let update_device = UpdateDevice::from_primitive(device);
        let device = self.database.devices().update_device(update_device)?;
        Ok(device)
    }

    pub async fn send_push_notification_device(&mut self, device_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let device = self.get_device(device_id)?;
        let notification = GorushNotification::new(
            vec![device.token],
            device.platform,
            "Test Notification".to_string(),
            "Test Message".to_string(),
            PushNotification {
                notification_type: PushNotificationTypes::Test,
                data: None,
            },
        );
        Ok(self.pusher.push_notifications(vec![notification]).await?.counts > 0)
    }

    pub fn delete_device(&mut self, device_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.database.devices().delete_device(device_id)
    }
}
