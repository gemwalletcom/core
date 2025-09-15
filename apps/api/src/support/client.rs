use primitives::SupportDevice;
use storage::{DatabaseClient, DatabaseError};

pub struct SupportClient {
    database: DatabaseClient,
}

impl SupportClient {
    pub fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub fn add_support_device(&mut self, support_id: &str, device_id: &str) -> Result<SupportDevice, DatabaseError> {
        let device = self.database.get_device(device_id)?;
        self.database.support().add_support_device(support_id, device.id)
    }
}
