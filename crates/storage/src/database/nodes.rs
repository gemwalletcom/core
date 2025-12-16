use crate::{DatabaseClient, models::*};
use diesel::prelude::*;

pub(crate) trait NodesStore {
    fn get_nodes(&mut self) -> Result<Vec<NodeRow>, diesel::result::Error>;
}

impl NodesStore for DatabaseClient {
    fn get_nodes(&mut self) -> Result<Vec<NodeRow>, diesel::result::Error> {
        use crate::schema::nodes::dsl::*;
        nodes.select(NodeRow::as_select()).load(&mut self.connection)
    }
}

// Public methods for backward compatibility
impl DatabaseClient {
    pub fn get_nodes(&mut self) -> Result<Vec<NodeRow>, diesel::result::Error> {
        NodesStore::get_nodes(self)
    }
}
