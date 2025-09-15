use crate::database::support::SupportStore;
use crate::models::Support;
use crate::{DatabaseClient, DatabaseError};

pub trait SupportRepository {
    fn add_support_device(&mut self, support_id: &str, device_id: i32) -> Result<primitives::SupportDevice, DatabaseError>;
    fn get_support_by_support_id(&mut self, support_id: &str) -> Result<primitives::SupportDevice, DatabaseError>;
}

impl SupportRepository for DatabaseClient {
    fn add_support_device(&mut self, support_id: &str, device_id: i32) -> Result<primitives::SupportDevice, DatabaseError> {
        Ok(SupportStore::add_support_device(
            self,
            Support {
                support_id: support_id.to_string(),
                device_id,
            },
        )?
        .as_primitive())
    }

    fn get_support_by_support_id(&mut self, support_id: &str) -> Result<primitives::SupportDevice, DatabaseError> {
        Ok(SupportStore::get_support_by_support_id(self, support_id)?.as_primitive())
    }
}
