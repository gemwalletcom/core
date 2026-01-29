use crate::{DatabaseClient, models::*};
use diesel::prelude::*;

pub trait DeviceSessionsStore {
    fn add_device_session(&mut self, session: NewDeviceSessionRow) -> Result<DeviceSessionRow, diesel::result::Error>;
}

impl DeviceSessionsStore for DatabaseClient {
    fn add_device_session(&mut self, session: NewDeviceSessionRow) -> Result<DeviceSessionRow, diesel::result::Error> {
        use crate::schema::devices_sessions::dsl::*;
        diesel::insert_into(devices_sessions)
            .values(&session)
            .returning(DeviceSessionRow::as_returning())
            .get_result(&mut self.connection)
    }
}
