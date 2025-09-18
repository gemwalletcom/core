use crate::database::support::SupportStore;
use crate::models::Support;
use crate::{DatabaseClient, DatabaseError};

pub trait SupportRepository {
    fn add_support_device(&mut self, support_id: &str, device_id: &str) -> Result<primitives::SupportDevice, DatabaseError>;
    fn get_support(&mut self, support_id: &str) -> Result<primitives::SupportDevice, DatabaseError>;
    fn support_update_unread(&mut self, support_id: &str, unread: i32) -> Result<primitives::SupportDevice, DatabaseError>;
}

impl SupportRepository for DatabaseClient {
    fn add_support_device(&mut self, support_id: &str, device_id: &str) -> Result<primitives::SupportDevice, DatabaseError> {
        let device = self.get_device(device_id)?;
        let support = SupportStore::add_support_device(
            self,
            Support {
                support_id: support_id.to_string(),
                device_id: device.id,
                unread: 0,
            },
        )?;

        Ok(support.as_primitive(device.device_id))
    }

    fn get_support(&mut self, support_id_value: &str) -> Result<primitives::SupportDevice, DatabaseError> {
        let support = SupportStore::get_support(self, support_id_value)?;
        let device = self.get_device_by_id(support.device_id)?;
        Ok(support.as_primitive(device.device_id))
    }

    fn support_update_unread(&mut self, support_id_value: &str, unread: i32) -> Result<primitives::SupportDevice, DatabaseError> {
        let support = SupportStore::support_update_unread(self, support_id_value, unread)?;
        let device = self.get_device_by_id(support.device_id)?;
        Ok(support.as_primitive(device.device_id))
    }
}
