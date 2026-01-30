use api_connector::PusherClient;
use primitives::{Device, GorushNotification, PushNotification, PushNotificationTypes};
use std::error::Error;
use storage::{Database, DevicesRepository, models::UpdateDeviceRow};

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
        let add_device = UpdateDeviceRow::from_primitive(device.clone());
        Ok(self.database.devices()?.add_device(add_device)?)
    }

    pub fn get_device(&self, device_id: &str) -> Result<Device, Box<dyn Error + Send + Sync>> {
        Ok(self.database.devices()?.get_device(device_id)?)
    }

    pub fn update_device(&self, device: Device) -> Result<Device, Box<dyn Error + Send + Sync>> {
        let update_device = UpdateDeviceRow::from_primitive(device);
        Ok(self.database.devices()?.update_device(update_device)?)
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
        Ok(self.database.devices()?.delete_device(device_id)?)
    }

    pub fn is_device_registered(&self, device_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self.database.devices()?.get_device_exist(device_id)?)
    }
}
