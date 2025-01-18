use crate::{models::*, DatabaseClient};

use diesel::prelude::*;

impl DatabaseClient {
    pub fn add_link_types(&mut self, values: Vec<LinkType>) -> Result<usize, diesel::result::Error> {
        use crate::schema::link_types::dsl::*;
        diesel::insert_into(link_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}
