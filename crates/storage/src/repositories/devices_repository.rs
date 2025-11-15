use crate::database::devices::{DeviceFieldUpdate, DeviceFilter, DevicesStore};
use crate::{DatabaseClient, DatabaseError};

pub trait DevicesRepository {
    fn add_device(&mut self, device: crate::models::UpdateDevice) -> Result<primitives::Device, DatabaseError>;
    fn get_device_by_id(&mut self, id: i32) -> Result<primitives::Device, DatabaseError>;
    fn get_device(&mut self, device_id: &str) -> Result<primitives::Device, DatabaseError>;
    fn update_device(&mut self, device: crate::models::UpdateDevice) -> Result<primitives::Device, DatabaseError>;
    fn update_device_fields(&mut self, device_ids: Vec<String>, updates: Vec<DeviceFieldUpdate>) -> Result<usize, DatabaseError>;
    fn delete_device(&mut self, device_id: &str) -> Result<usize, DatabaseError>;
    fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, DatabaseError>;
    fn devices_inactive_days(&mut self, min_days: i64, max_days: i64, push_enabled: Option<bool>) -> Result<Vec<primitives::Device>, DatabaseError>;
}

impl DevicesRepository for DatabaseClient {
    fn add_device(&mut self, device: crate::models::UpdateDevice) -> Result<primitives::Device, DatabaseError> {
        Ok(DevicesStore::add_device(self, device)?.as_primitive())
    }

    fn get_device_by_id(&mut self, id: i32) -> Result<primitives::Device, DatabaseError> {
        Ok(DevicesStore::get_device_by_id(self, id)?.as_primitive())
    }

    fn get_device(&mut self, device_id: &str) -> Result<primitives::Device, DatabaseError> {
        Ok(DevicesStore::get_device(self, device_id)?.as_primitive())
    }

    fn update_device(&mut self, device: crate::models::UpdateDevice) -> Result<primitives::Device, DatabaseError> {
        Ok(DevicesStore::update_device(self, device)?.as_primitive())
    }

    fn update_device_fields(&mut self, device_ids: Vec<String>, updates: Vec<DeviceFieldUpdate>) -> Result<usize, DatabaseError> {
        Ok(DevicesStore::update_device_fields(self, device_ids, updates)?)
    }

    fn delete_device(&mut self, device_id: &str) -> Result<usize, DatabaseError> {
        Ok(DevicesStore::delete_device(self, device_id)?)
    }

    fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, DatabaseError> {
        Ok(DevicesStore::delete_devices_subscriptions_after_days(self, days)?)
    }

    fn devices_inactive_days(&mut self, min_days: i64, max_days: i64, push_enabled: Option<bool>) -> Result<Vec<primitives::Device>, DatabaseError> {
        use chrono::{Duration, Utc};

        let min_days_cutoff = Utc::now() - Duration::days(min_days);
        let max_days_cutoff = Utc::now() - Duration::days(max_days);

        let mut filters = vec![DeviceFilter::CreatedBetween {
            start: max_days_cutoff.naive_utc(),
            end: min_days_cutoff.naive_utc(),
        }];

        if let Some(enabled) = push_enabled {
            filters.push(DeviceFilter::IsPushEnabled(enabled));
        }

        let result = DevicesStore::get_devices_by_filter(self, filters)?;
        Ok(result.into_iter().map(|x| x.as_primitive()).collect())
    }
}
