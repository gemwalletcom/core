use crate::{DatabaseClient, models::ChainIdRow};
use diesel::prelude::*;
use primitives::Chain;

pub trait ChainStore {
    fn add_chains(&mut self, values: Vec<Chain>) -> Result<usize, diesel::result::Error>;
}

impl ChainStore for DatabaseClient {
    fn add_chains(&mut self, values: Vec<Chain>) -> Result<usize, diesel::result::Error> {
        let chain_values = values.into_iter().map(|chain_id| ChainIdRow { id: chain_id.into() }).collect::<Vec<_>>();
        use crate::schema::chains::dsl::*;
        diesel::insert_into(chains).values(chain_values).on_conflict_do_nothing().execute(&mut self.connection)
    }
}
