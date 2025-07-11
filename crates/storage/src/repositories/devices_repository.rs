use std::error::Error;

use crate::database::devices::DevicesStore;
use crate::DatabaseClient;

pub trait DevicesRepository {
    fn add_device(&mut self, device: crate::models::UpdateDevice) -> Result<primitives::Device, Box<dyn Error + Send + Sync>>;
    fn get_device_by_id(&mut self, id: i32) -> Result<primitives::Device, Box<dyn Error + Send + Sync>>;
    fn get_device(&mut self, device_id: &str) -> Result<primitives::Device, Box<dyn Error + Send + Sync>>;
    fn update_device(&mut self, device: crate::models::UpdateDevice) -> Result<primitives::Device, Box<dyn Error + Send + Sync>>;
    fn delete_device(&mut self, device_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn update_device_is_push_enabled(&mut self, device_id: &str, value: bool) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn devices_inactive_days(
        &mut self,
        min_days: i64,
        max_days: i64,
        push_enabled: Option<bool>,
    ) -> Result<Vec<primitives::Device>, Box<dyn Error + Send + Sync>>;
}

impl DevicesRepository for DatabaseClient {
    fn add_device(&mut self, device: crate::models::UpdateDevice) -> Result<primitives::Device, Box<dyn Error + Send + Sync>> {
        let result = DevicesStore::add_device(self, device)?;
        Ok(result.as_primitive())
    }

    fn get_device_by_id(&mut self, id: i32) -> Result<primitives::Device, Box<dyn Error + Send + Sync>> {
        let result = DevicesStore::get_device_by_id(self, id)?;
        Ok(result.as_primitive())
    }

    fn get_device(&mut self, device_id: &str) -> Result<primitives::Device, Box<dyn Error + Send + Sync>> {
        let result = DevicesStore::get_device(self, device_id)?;
        Ok(result.as_primitive())
    }

    fn update_device(&mut self, device: crate::models::UpdateDevice) -> Result<primitives::Device, Box<dyn Error + Send + Sync>> {
        let result = DevicesStore::update_device(self, device)?;
        Ok(result.as_primitive())
    }

    fn delete_device(&mut self, device_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(DevicesStore::delete_device(self, device_id)?)
    }

    fn update_device_is_push_enabled(&mut self, device_id: &str, value: bool) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(DevicesStore::update_device_is_push_enabled(self, device_id, value)?)
    }

    fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(DevicesStore::delete_devices_subscriptions_after_days(self, days)?)
    }

    fn devices_inactive_days(
        &mut self,
        min_days: i64,
        max_days: i64,
        push_enabled: Option<bool>,
    ) -> Result<Vec<primitives::Device>, Box<dyn Error + Send + Sync>> {
        let result = DevicesStore::devices_inactive_days(self, min_days, max_days, push_enabled)?;
        Ok(result.into_iter().map(|x| x.as_primitive()).collect())
    }
}
