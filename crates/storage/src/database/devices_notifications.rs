use crate::DatabaseClient;
use crate::models::{DeviceNotificationRow, NewDeviceNotificationRow};
use crate::sql_types::PushNotificationType;
use diesel::prelude::*;

pub trait DevicesNotificationsStore {
    fn add_device_notification(&mut self, device_id: i32, notification_type: PushNotificationType, error: Option<String>) -> Result<DeviceNotificationRow, diesel::result::Error>;
    fn get_device_notifications(&mut self, device_id: i32) -> Result<Vec<DeviceNotificationRow>, diesel::result::Error>;
}

impl DevicesNotificationsStore for DatabaseClient {
    fn add_device_notification(&mut self, device_id: i32, notification_type: PushNotificationType, error: Option<String>) -> Result<DeviceNotificationRow, diesel::result::Error> {
        use crate::schema::devices_notifications::dsl;

        let row = NewDeviceNotificationRow {
            device_id,
            notification_type,
            error,
        };

        diesel::insert_into(dsl::devices_notifications)
            .values(&row)
            .returning(DeviceNotificationRow::as_returning())
            .get_result(&mut self.connection)
    }

    fn get_device_notifications(&mut self, device_id: i32) -> Result<Vec<DeviceNotificationRow>, diesel::result::Error> {
        use crate::schema::devices_notifications::dsl;

        dsl::devices_notifications
            .filter(dsl::device_id.eq(device_id))
            .order(dsl::created_at.desc())
            .select(DeviceNotificationRow::as_select())
            .load(&mut self.connection)
    }
}
