use std::error::Error;
use storage::database::DatabaseClient;

pub struct DeviceUpdater {
    database: DatabaseClient,
}

impl DeviceUpdater {
    pub fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub async fn update(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.delete_devices_subscriptions_after_days(120)?)
    }
}
