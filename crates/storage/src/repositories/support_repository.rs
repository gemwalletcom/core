use primitives::SupportDevice;

use crate::database::support::SupportStore;
use crate::models::Support;
use crate::{DatabaseClient, DatabaseError};

pub trait SupportRepository {
    fn add_support_device(&mut self, support_id: &str, device_id: &str) -> Result<primitives::SupportDevice, DatabaseError>;
}

impl SupportRepository for DatabaseClient {
    fn add_support_device(&mut self, support_id: &str, device_id: &str) -> Result<primitives::SupportDevice, DatabaseError> {
        let device = self.get_device(device_id)?;
        SupportStore::add_support_device(
            self,
            Support {
                support_id: support_id.to_string(),
                device_id: device.id,
            },
        )?;

        Ok(SupportDevice {
            support_id: support_id.to_string(),
            device_id: device.device_id.to_string(),
        })
    }
}
