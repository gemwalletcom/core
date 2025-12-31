use crate::database::chains::ChainStore;
use crate::{DatabaseClient, DatabaseError};

pub trait ChainsRepository {
    fn add_chains(&mut self, chains: Vec<String>) -> Result<usize, DatabaseError>;
}

impl ChainsRepository for DatabaseClient {
    fn add_chains(&mut self, chains: Vec<String>) -> Result<usize, DatabaseError> {
        Ok(ChainStore::add_chains(self, chains)?)
    }
}
