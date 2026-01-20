use crate::{DatabaseClient, models::ChainRow};
use diesel::prelude::*;

pub trait ChainStore {
    fn add_chains(&mut self, values: Vec<String>) -> Result<usize, diesel::result::Error>;
}

impl ChainStore for DatabaseClient {
    fn add_chains(&mut self, values: Vec<String>) -> Result<usize, diesel::result::Error> {
        let chain_values = values.iter().map(|chain_id| ChainRow { id: chain_id.clone() }).collect::<Vec<_>>();
        use crate::schema::chains::dsl::*;
        diesel::insert_into(chains).values(chain_values).on_conflict_do_nothing().execute(&mut self.connection)
    }
}
