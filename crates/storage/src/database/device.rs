use crate::{models::*, DatabaseClient};
use chrono::{Duration, Utc};
use diesel::{prelude::*, upsert::excluded};

impl DatabaseClient {
    pub fn add_device(&mut self, device: UpdateDevice) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::insert_into(devices)
            .values(&device)
            .on_conflict(device_id)
            .do_update()
            .set((device_id.eq(excluded(device_id)),))
            .returning(Device::as_returning())
            .get_result(&mut self.connection)
    }

    pub fn get_device_by_id(&mut self, _id: i32) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        devices.filter(id.eq(_id)).select(Device::as_select()).first(&mut self.connection)
    }

    pub fn get_device(&mut self, _device_id: &str) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        devices.filter(device_id.eq(_device_id)).select(Device::as_select()).first(&mut self.connection)
    }

    pub fn get_device_token(&mut self, _device_id: &str) -> Result<String, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        devices.filter(device_id.eq(_device_id)).select(token).first(&mut self.connection)
    }

    pub fn update_device(&mut self, device: UpdateDevice) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::update(devices)
            .filter(device_id.eq(device.clone().device_id))
            .set(device)
            .returning(Device::as_returning())
            .get_result(&mut self.connection)
    }

    pub fn delete_device(&mut self, _device_id: &str) -> Result<usize, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::delete(devices.filter(device_id.eq(_device_id))).execute(&mut self.connection)
    }

    pub fn update_device_is_push_enabled(&mut self, _device_id: &str, value: bool) -> Result<usize, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::update(devices)
            .filter(device_id.eq(_device_id))
            .set(is_push_enabled.eq(value))
            .execute(&mut self.connection)
    }

    // Delete subscriptions for inactive devices
    pub fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, diesel::result::Error> {
        let cutoff_date = Utc::now() - Duration::days(days);

        let device_ids_query = crate::schema::devices::table
            .filter(crate::schema::devices::updated_at.lt(cutoff_date.naive_utc()))
            .select(crate::schema::devices::id);

        diesel::delete(crate::schema::subscriptions::table.filter(crate::schema::subscriptions::device_id.eq_any(device_ids_query)))
            .execute(&mut self.connection)
    }
}
