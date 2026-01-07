use crate::{DatabaseClient, models::*};
use diesel::prelude::*;

pub trait SupportStore {
    fn add_support_device(&mut self, value: SupportRow) -> Result<SupportRow, diesel::result::Error>;
    fn get_support_device(&mut self, support_id_param: &str) -> Result<DeviceRow, diesel::result::Error>;
    fn get_support(&mut self, support_id_param: &str) -> Result<SupportRow, diesel::result::Error>;
    fn support_update_unread(&mut self, support_id_param: &str, unread_value: i32) -> Result<SupportRow, diesel::result::Error>;
}

impl SupportStore for DatabaseClient {
    fn add_support_device(&mut self, value: SupportRow) -> Result<SupportRow, diesel::result::Error> {
        use crate::schema::support::dsl::*;

        diesel::insert_into(support)
            .values(&value)
            .on_conflict(support_id)
            .do_update()
            .set(device_id.eq(value.device_id))
            .returning(SupportRow::as_returning())
            .get_result(&mut self.connection)
    }

    fn get_support_device(&mut self, support_device_id: &str) -> Result<DeviceRow, diesel::result::Error> {
        use crate::schema::{devices, support};
        support::table
            .inner_join(devices::table)
            .filter(support::support_id.eq(support_device_id))
            .select(DeviceRow::as_select())
            .first(&mut self.connection)
    }

    fn get_support(&mut self, support_device_id: &str) -> Result<SupportRow, diesel::result::Error> {
        use crate::schema::support::dsl::*;
        support
            .filter(support_id.eq(support_device_id))
            .select(SupportRow::as_select())
            .first(&mut self.connection)
    }

    fn support_update_unread(&mut self, support_device_id: &str, unread_value: i32) -> Result<SupportRow, diesel::result::Error> {
        use crate::schema::support::dsl::*;
        diesel::update(support.filter(support_id.eq(support_device_id)))
            .set(unread.eq(unread_value))
            .returning(SupportRow::as_returning())
            .get_result(&mut self.connection)
    }
}
