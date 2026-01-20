use crate::database::notifications::NotificationsStore;
use crate::models::NewNotificationRow;
use crate::{DatabaseClient, DatabaseError};
use chrono::NaiveDateTime;
use primitives::NotificationData;

pub trait NotificationsRepository {
    fn get_notifications_by_device_id(&mut self, device_id: &str, from_datetime: Option<NaiveDateTime>) -> Result<Vec<NotificationData>, DatabaseError>;
    fn create_notifications(&mut self, notifications: Vec<NewNotificationRow>) -> Result<usize, DatabaseError>;
    fn mark_all_as_read(&mut self, device_id: &str) -> Result<usize, DatabaseError>;
}

impl NotificationsRepository for DatabaseClient {
    fn get_notifications_by_device_id(&mut self, device_id: &str, from_datetime: Option<NaiveDateTime>) -> Result<Vec<NotificationData>, DatabaseError> {
        Ok(NotificationsStore::get_notifications_by_device_id(self, device_id, from_datetime)?
            .into_iter()
            .map(|(row, wallet_identifier)| row.as_primitive(wallet_identifier))
            .collect())
    }

    fn create_notifications(&mut self, notifications: Vec<NewNotificationRow>) -> Result<usize, DatabaseError> {
        NotificationsStore::create_notifications(self, notifications)
    }

    fn mark_all_as_read(&mut self, device_id: &str) -> Result<usize, DatabaseError> {
        NotificationsStore::mark_all_as_read(self, device_id)
    }
}
