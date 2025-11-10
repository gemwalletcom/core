use primitives::SupportDevice;
use storage::Database;
use std::error::Error;

#[derive(Clone)]
pub struct SupportClient {
    database: Database,
}

impl SupportClient {
    pub fn new(database: Database) -> Self {
        
        Self { database }
    }

    pub fn add_support_device(&self, support_id: &str, device_id: &str) -> Result<SupportDevice, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.support().add_support_device(support_id, device_id)?)
    }

    pub fn get_support_device(&self, support_id: &str) -> Result<SupportDevice, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.support().get_support(support_id)?)
    }
}
