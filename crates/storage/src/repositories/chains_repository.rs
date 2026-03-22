use crate::database::chains::ChainStore;
use crate::{DatabaseClient, DatabaseError};
use primitives::Chain;

pub trait ChainsRepository {
    fn add_chains(&mut self, chains: Vec<Chain>) -> Result<usize, DatabaseError>;
}

impl ChainsRepository for DatabaseClient {
    fn add_chains(&mut self, chains: Vec<Chain>) -> Result<usize, DatabaseError> {
        Ok(ChainStore::add_chains(self, chains)?)
    }
}
