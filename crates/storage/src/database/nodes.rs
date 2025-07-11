use crate::{models::*, DatabaseClient};
use diesel::prelude::*;

pub(crate) trait NodesStore {
    fn get_nodes(&mut self) -> Result<Vec<Node>, diesel::result::Error>;
}

impl NodesStore for DatabaseClient {
    fn get_nodes(&mut self) -> Result<Vec<Node>, diesel::result::Error> {
        use crate::schema::nodes::dsl::*;
        nodes.select(Node::as_select()).load(&mut self.connection)
    }
}

// Public methods for backward compatibility
impl DatabaseClient {
    pub fn get_nodes(&mut self) -> Result<Vec<Node>, diesel::result::Error> {
        NodesStore::get_nodes(self)
    }
}
