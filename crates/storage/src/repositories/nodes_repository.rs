use crate::DatabaseError;

use crate::database::nodes::NodesStore;
use crate::DatabaseClient;
use primitives::node::ChainNode;

pub trait NodesRepository {
    fn get_nodes(&mut self) -> Result<Vec<ChainNode>, DatabaseError>;
}

impl NodesRepository for DatabaseClient {
    fn get_nodes(&mut self) -> Result<Vec<ChainNode>, DatabaseError> {
        let result = NodesStore::get_nodes(self)?;
        Ok(result.into_iter().map(|x| x.as_primitive()).collect())
    }
}
