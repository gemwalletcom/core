use crate::database::subscriptions::SubscriptionsStore;
use crate::{DatabaseClient, models::*};
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::{prelude::*, upsert::excluded};

#[derive(Debug, Clone)]
pub enum DeviceFieldUpdate {
    IsPushEnabled(bool),
    IsPriceAlertsEnabled(bool),
}

#[derive(Debug, Clone)]
pub enum DeviceFilter {
    IsPushEnabled(bool),
    CreatedBetween { start: NaiveDateTime, end: NaiveDateTime },
}

pub(crate) trait DevicesStore {
    fn add_device(&mut self, device: UpdateDevice) -> Result<Device, diesel::result::Error>;
    fn get_device_by_id(&mut self, id: i32) -> Result<Device, diesel::result::Error>;
    fn get_device(&mut self, device_id: &str) -> Result<Device, diesel::result::Error>;
    fn update_device(&mut self, device: UpdateDevice) -> Result<Device, diesel::result::Error>;
    fn update_device_fields(&mut self, device_ids: Vec<String>, updates: Vec<DeviceFieldUpdate>) -> Result<usize, diesel::result::Error>;
    fn delete_device(&mut self, device_id: &str) -> Result<usize, diesel::result::Error>;
    fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, diesel::result::Error>;
    fn get_devices_by_filter(&mut self, filters: Vec<DeviceFilter>) -> Result<Vec<Device>, diesel::result::Error>;
}

impl DevicesStore for DatabaseClient {
    fn add_device(&mut self, device: UpdateDevice) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::insert_into(devices)
            .values(&device)
            .on_conflict(device_id)
            .do_update()
            .set((device_id.eq(excluded(device_id)),))
            .returning(Device::as_returning())
            .get_result(&mut self.connection)
    }

    fn get_device_by_id(&mut self, _id: i32) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        devices.find(_id).select(Device::as_select()).first(&mut self.connection)
    }

    fn get_device(&mut self, _device_id: &str) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        devices.filter(device_id.eq(_device_id)).select(Device::as_select()).first(&mut self.connection)
    }

    fn update_device(&mut self, device: UpdateDevice) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::update(devices)
            .filter(device_id.eq(device.clone().device_id))
            .set(device)
            .returning(Device::as_returning())
            .get_result(&mut self.connection)
    }

    fn update_device_fields(&mut self, device_ids: Vec<String>, updates: Vec<DeviceFieldUpdate>) -> Result<usize, diesel::result::Error> {
        use crate::schema::devices::dsl::*;

        if updates.is_empty() || device_ids.is_empty() {
            return Ok(0);
        }

        let mut total_updated = 0;
        for update in updates {
            let target = devices.filter(device_id.eq_any(&device_ids));
            let updated = match update {
                DeviceFieldUpdate::IsPushEnabled(value) => diesel::update(target).set(is_push_enabled.eq(value)).execute(&mut self.connection)?,
                DeviceFieldUpdate::IsPriceAlertsEnabled(value) => {
                    diesel::update(target).set(is_price_alerts_enabled.eq(value)).execute(&mut self.connection)?
                }
            };
            total_updated += updated;
        }

        Ok(total_updated)
    }

    fn delete_device(&mut self, _device_id: &str) -> Result<usize, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::delete(devices.filter(device_id.eq(_device_id))).execute(&mut self.connection)
    }

    // Delete subscriptions for inactive devices
    fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        let cutoff_date = Utc::now() - Duration::days(days);
        let device_ids: Vec<i32> = devices.filter(updated_at.lt(cutoff_date.naive_utc())).select(id).load(&mut self.connection)?;
        SubscriptionsStore::delete_subscriptions_for_device_ids(self, device_ids)
    }

    fn get_devices_by_filter(&mut self, filters: Vec<DeviceFilter>) -> Result<Vec<Device>, diesel::result::Error> {
        use crate::schema::devices::dsl::*;

        let mut query = devices.into_boxed();

        for filter in filters {
            match filter {
                DeviceFilter::IsPushEnabled(enabled) => {
                    query = query.filter(is_push_enabled.eq(enabled));
                }
                DeviceFilter::CreatedBetween { start, end } => {
                    query = query.filter(
                        created_at
                            .between(start, end)
                            .and(diesel::dsl::sql::<diesel::sql_types::Bool>(
                                "DATE_TRUNC('hour', updated_at) = DATE_TRUNC('hour', created_at)",
                            )),
                    );
                }
            }
        }

        query.select(Device::as_select()).load(&mut self.connection)
    }
}

// Public methods for backward compatibility
impl DatabaseClient {
    pub fn get_device(&mut self, device_id: &str) -> Result<Device, diesel::result::Error> {
        DevicesStore::get_device(self, device_id)
    }

    pub fn get_device_by_id(&mut self, id: i32) -> Result<Device, diesel::result::Error> {
        DevicesStore::get_device_by_id(self, id)
    }

    pub fn add_device(&mut self, device: UpdateDevice) -> Result<Device, diesel::result::Error> {
        DevicesStore::add_device(self, device)
    }

    pub fn update_device(&mut self, device: UpdateDevice) -> Result<Device, diesel::result::Error> {
        DevicesStore::update_device(self, device)
    }

    pub fn delete_device(&mut self, device_id: &str) -> Result<usize, diesel::result::Error> {
        DevicesStore::delete_device(self, device_id)
    }

    pub fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, diesel::result::Error> {
        DevicesStore::delete_devices_subscriptions_after_days(self, days)
    }
}
