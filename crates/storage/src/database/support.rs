use crate::{models::*, DatabaseClient};
use diesel::prelude::*;

pub(crate) trait SupportStore {
    fn add_support_device(&mut self, value: Support) -> Result<Support, diesel::result::Error>;
    fn get_support_device(&mut self, support_id_param: &str) -> Result<Device, diesel::result::Error>;
}

impl SupportStore for DatabaseClient {
    fn add_support_device(&mut self, value: Support) -> Result<Support, diesel::result::Error> {
        use crate::schema::support::dsl::*;

        diesel::insert_into(support)
            .values(&value)
            .on_conflict(support_id)
            .do_update()
            .set(&value)
            .returning(Support::as_returning())
            .get_result(&mut self.connection)
    }

    fn get_support_device(&mut self, support_device_id: &str) -> Result<Device, diesel::result::Error> {
        use crate::schema::{devices, support};
        support::table
            .inner_join(devices::table)
            .filter(support::support_id.eq(support_device_id))
            .select(Device::as_select())
            .first(&mut self.connection)
    }
}

impl DatabaseClient {
    pub fn add_support_device(&mut self, support_id: &str, device_id: i32) -> Result<Support, diesel::result::Error> {
        SupportStore::add_support_device(
            self,
            Support {
                support_id: support_id.to_string(),
                device_id,
            },
        )
    }

    pub fn get_support_device(&mut self, support_device_id: &str) -> Result<Device, diesel::result::Error> {
        SupportStore::get_support_device(self, support_device_id)
    }
}
