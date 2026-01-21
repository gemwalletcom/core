use chrono::{DateTime, Utc};
use in_app_notifications::map_notification;
use localizer::LanguageLocalizer;
use primitives::InAppNotification;
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

    pub fn get_notifications(&self, device_id: &str, from_timestamp: Option<u64>) -> Result<Vec<InAppNotification>, Box<dyn Error + Send + Sync>> {
        let localizer = LanguageLocalizer::new();
        let from_datetime = from_timestamp.and_then(|ts| DateTime::<Utc>::from_timestamp(ts as i64, 0).map(|dt| dt.naive_utc()));
        let notifications = self.database.notifications()?.get_notifications_by_device_id(device_id, from_datetime)?;
        Ok(notifications.into_iter().map(|n| map_notification(n, &localizer)).collect())
    }

    pub fn mark_all_as_read(&self, device_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.notifications()?.mark_all_as_read(device_id)?)
    }
}
