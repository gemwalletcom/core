use crate::{models::*, DatabaseClient};

use diesel::prelude::*;

pub trait LinkStore {
    fn add_link_types(&mut self, values: Vec<LinkType>) -> Result<usize, diesel::result::Error>;
}

pub trait LinkRepository {
    fn add_link_types(&mut self, values: Vec<primitives::LinkType>) -> Result<usize, Box<dyn std::error::Error>>;
}

impl LinkStore for DatabaseClient {
    fn add_link_types(&mut self, values: Vec<LinkType>) -> Result<usize, diesel::result::Error> {
        use crate::schema::link_types::dsl::*;
        diesel::insert_into(link_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}

impl LinkRepository for DatabaseClient {
    fn add_link_types(&mut self, values: Vec<primitives::LinkType>) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(LinkStore::add_link_types(self, values.into_iter().map(LinkType::from_primitive).collect())?)
    }
}
