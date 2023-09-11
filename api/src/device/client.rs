extern crate rocket;
use std::error::Error;

use storage::{DatabaseClient, models::UpdateDevice};

pub struct DevicesClient {
    database: DatabaseClient,
}

impl DevicesClient {
    pub async fn new(
        database_url: &str
    ) -> Self {
        let database = DatabaseClient::new(database_url);
        Self {
            database,
        }
    }

    pub fn add_device(&mut self, device: primitives::device::Device) -> Result<primitives::device::Device, Box<dyn Error>> {
        let add_device = UpdateDevice::from_primitive(device.clone());
        let device = self.database.add_device(add_device)?;
        Ok(device.as_primitive())
    }

    pub fn get_device(&mut self, device_id: &str) -> Result<primitives::Device, Box<dyn Error>> {
        let device = self.database.get_device(device_id)?;
        Ok(device.as_primitive())
    }
    pub fn update_device(&mut self, device: primitives::device::Device) -> Result<primitives::device::Device, Box<dyn Error>> {
        let update_device = UpdateDevice::from_primitive(device);
        let device = self.database.update_device(update_device)?;
        Ok(device.as_primitive())
    }

    pub fn delete_device(&mut self, device_id: &str) -> Result<usize, Box<dyn Error>> {
        Ok(self.database.delete_device(device_id)?)
    }
}