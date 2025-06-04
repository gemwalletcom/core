use crate::{models::*, DatabaseClient};
use diesel::prelude::*;
use primitives::node::ChainNode;

pub trait NodeStore {
    fn get_nodes(&mut self) -> Result<Vec<Node>, diesel::result::Error>;
}

pub trait NodeRepository {
    fn get_nodes(&mut self) -> Result<Vec<ChainNode>, Box<dyn std::error::Error>>;
}

impl NodeStore for DatabaseClient {
    fn get_nodes(&mut self) -> Result<Vec<Node>, diesel::result::Error> {
        use crate::schema::nodes::dsl::*;
        nodes.select(Node::as_select()).load(&mut self.connection)
    }
}

impl NodeRepository for DatabaseClient {
    fn get_nodes(&mut self) -> Result<Vec<ChainNode>, Box<dyn std::error::Error>> {
        Ok(NodeStore::get_nodes(self)?.into_iter().map(|x| x.as_primitive()).collect())
    }
}
