use crate::database::support::SupportStore;
use crate::models::SupportRow;
use crate::{DatabaseClient, DatabaseError};
use primitives::SupportDevice;

pub trait SupportRepository {
    fn add_support_device(&mut self, support_id: &str, device_id: &str) -> Result<SupportDevice, DatabaseError>;
    fn get_support(&mut self, support_device_id: &str) -> Result<SupportDevice, DatabaseError>;
    fn support_update_unread(&mut self, support_device_id: &str, unread: i32) -> Result<SupportDevice, DatabaseError>;
}

impl SupportRepository for DatabaseClient {
    fn add_support_device(&mut self, support_id: &str, device_id: &str) -> Result<SupportDevice, DatabaseError> {
        let device = self.get_device(device_id)?;
        let support = SupportStore::add_support_device(
            self,
            SupportRow {
                support_id: support_id.to_string(),
                device_id: device.id,
                unread: 0,
            },
        )?;

        Ok(support.as_primitive())
    }

    fn get_support(&mut self, support_device_id: &str) -> Result<SupportDevice, DatabaseError> {
        let support = SupportStore::get_support(self, support_device_id)?;
        Ok(support.as_primitive())
    }

    fn support_update_unread(&mut self, support_device_id: &str, unread: i32) -> Result<SupportDevice, DatabaseError> {
        let support = SupportStore::support_update_unread(self, support_device_id, unread)?;
        Ok(support.as_primitive())
    }
}
