use std::error::Error;
use storage::Database;

pub struct DeviceUpdater {
    database: Database,
}

impl DeviceUpdater {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn update(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.devices().delete_devices_subscriptions_after_days(120)?)
    }
}
