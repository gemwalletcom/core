use api_connector::PusherClient;
use primitives::{Device, GorushNotification, PushNotification, PushNotificationTypes};
use std::error::Error;
use storage::{Database, models::UpdateDevice};

#[derive(Clone)]
pub struct DevicesClient {
    database: Database,
    pusher: PusherClient,
}

impl DevicesClient {
    pub fn new(database: Database, pusher: PusherClient) -> Self {
        Self { database, pusher }
    }

    pub fn add_device(&self, device: Device) -> Result<Device, Box<dyn Error + Send + Sync>> {
        let add_device = UpdateDevice::from_primitive(device.clone());
        let device = self.database.client()?.devices().add_device(add_device)?;
        Ok(device)
    }

    pub fn get_device(&self, device_id: &str) -> Result<Device, Box<dyn Error + Send + Sync>> {
        let device = self.database.client()?.devices().get_device(device_id)?;
        Ok(device)
    }

    pub fn update_device(&self, device: Device) -> Result<Device, Box<dyn Error + Send + Sync>> {
        let update_device = UpdateDevice::from_primitive(device);
        let device = self.database.client()?.devices().update_device(update_device)?;
        Ok(device)
    }

    pub async fn send_push_notification_device(&self, device_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let device = self.get_device(device_id)?;
        let notification = GorushNotification::from_device(
            device,
            "Test Notification".to_string(),
            "Test Message".to_string(),
            PushNotification {
                notification_type: PushNotificationTypes::Test,
                data: None,
            },
        );
        Ok(self.pusher.push_notifications(vec![notification]).await?.response.counts > 0)
    }

    pub fn delete_device(&self, device_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.devices().delete_device(device_id)?)
    }
}
