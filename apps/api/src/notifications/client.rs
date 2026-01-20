use primitives::Notification;
use std::error::Error;
use storage::{Database, NotificationsRepository};

#[derive(Clone)]
pub struct NotificationsClient {
    database: Database,
}

impl NotificationsClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn get_notifications(&self, device_id: &str) -> Result<Vec<Notification>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.notifications()?.get_notifications_by_device_id(device_id)?)
    }

    pub fn mark_all_as_read(&self, device_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.notifications()?.mark_all_as_read(device_id)?)
    }
}
