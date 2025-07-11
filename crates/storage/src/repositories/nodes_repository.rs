use std::error::Error;

use crate::DatabaseClient;
use crate::database::nodes::NodesStore;
use primitives::node::ChainNode;

pub trait NodesRepository {
    fn get_nodes(&mut self) -> Result<Vec<ChainNode>, Box<dyn Error + Send + Sync>>;
}

impl NodesRepository for DatabaseClient {
    fn get_nodes(&mut self) -> Result<Vec<ChainNode>, Box<dyn Error + Send + Sync>> {
        let result = NodesStore::get_nodes(self)?;
        Ok(result.into_iter().map(|x| x.as_primitive()).collect())
    }
}