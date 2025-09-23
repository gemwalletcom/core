use crate::{DatabaseClient, models::*};
use diesel::prelude::*;

pub(crate) trait LinkTypesStore {
    fn add_link_types(&mut self, values: Vec<LinkType>) -> Result<usize, diesel::result::Error>;
}

impl LinkTypesStore for DatabaseClient {
    fn add_link_types(&mut self, values: Vec<LinkType>) -> Result<usize, diesel::result::Error> {
        use crate::schema::link_types::dsl::*;
        diesel::insert_into(link_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}

// Public methods for backward compatibility
impl DatabaseClient {
    pub fn add_link_types(&mut self, values: Vec<LinkType>) -> Result<usize, diesel::result::Error> {
        LinkTypesStore::add_link_types(self, values)
    }
}
