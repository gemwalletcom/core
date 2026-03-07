use crate::database::devices_notifications::DevicesNotificationsStore;
use crate::models::DeviceNotificationRow;
use crate::sql_types::PushNotificationType;
use crate::{DatabaseClient, DatabaseError};

pub trait DevicesNotificationsRepository {
    fn add_device_notification(&mut self, device_id: i32, notification_type: PushNotificationType, error: Option<String>) -> Result<DeviceNotificationRow, DatabaseError>;
    fn get_device_notifications(&mut self, device_id: i32) -> Result<Vec<DeviceNotificationRow>, DatabaseError>;
}

impl DevicesNotificationsRepository for DatabaseClient {
    fn add_device_notification(&mut self, device_id: i32, notification_type: PushNotificationType, error: Option<String>) -> Result<DeviceNotificationRow, DatabaseError> {
        Ok(DevicesNotificationsStore::add_device_notification(self, device_id, notification_type, error)?)
    }

    fn get_device_notifications(&mut self, device_id: i32) -> Result<Vec<DeviceNotificationRow>, DatabaseError> {
        Ok(DevicesNotificationsStore::get_device_notifications(self, device_id)?)
    }
}
