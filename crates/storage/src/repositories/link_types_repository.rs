use crate::DatabaseError;

use crate::database::link_types::LinkTypesStore;
use crate::DatabaseClient;

pub trait LinkTypesRepository {
    fn add_link_types(&mut self, values: Vec<primitives::LinkType>) -> Result<usize, DatabaseError>;
}

impl LinkTypesRepository for DatabaseClient {
    fn add_link_types(&mut self, values: Vec<primitives::LinkType>) -> Result<usize, DatabaseError> {
        let storage_values = values.into_iter().map(crate::models::LinkType::from_primitive).collect();
        Ok(LinkTypesStore::add_link_types(self, storage_values)?)
    }
}
