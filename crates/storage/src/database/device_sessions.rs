use crate::{
    DatabaseClient,
    models::{DeviceSessionRow, NewDeviceSessionRow},
};
use diesel::prelude::*;

pub trait DeviceSessionsStore {
    fn add_device_session(&mut self, device_id: i32) -> Result<DeviceSessionRow, diesel::result::Error>;
}

impl DeviceSessionsStore for DatabaseClient {
    fn add_device_session(&mut self, device_row_id: i32) -> Result<DeviceSessionRow, diesel::result::Error> {
        use crate::schema::devices_sessions::dsl::*;

        let new_session = NewDeviceSessionRow { device_id: device_row_id };

        diesel::insert_into(devices_sessions)
            .values(&new_session)
            .returning(DeviceSessionRow::as_returning())
            .get_result(&mut self.connection)
    }
}
