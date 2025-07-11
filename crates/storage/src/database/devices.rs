use crate::{models::*, DatabaseClient};
use crate::database::subscriptions::SubscriptionsStore;
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
        use crate::schema::devices::dsl::*;
        let cutoff_date = Utc::now() - Duration::days(days);
        let device_ids: Vec<i32> = devices.filter(updated_at.lt(cutoff_date.naive_utc())).select(id).load(&mut self.connection)?;
        self.delete_subscriptions_for_device_ids(device_ids)
    }

    pub fn devices_inactive_days(&mut self, min_days: i64, max_days: i64, push_enabled: Option<bool>) -> Result<Vec<Device>, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        let min_days_cutoff = Utc::now() - Duration::days(min_days);
        let max_days_cutoff = Utc::now() - Duration::days(max_days);
        let mut query = devices.into_boxed();
        query = query.filter(
            created_at
                .between(max_days_cutoff.naive_utc(), min_days_cutoff.naive_utc())
                .and(diesel::dsl::sql::<diesel::sql_types::Bool>(
                    "DATE_TRUNC('hour', updated_at) = DATE_TRUNC('hour', created_at)",
                )),
        );
        if let Some(enabled) = push_enabled {
            query = query.filter(is_push_enabled.eq(enabled));
        }
        query.select(Device::as_select()).load(&mut self.connection)
    }
}
