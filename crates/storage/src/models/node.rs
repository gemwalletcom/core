use diesel::prelude::*;
use primitives::node::{ChainNode, NodeState};
use std::str::FromStr;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::nodes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Node {
    pub id: i32,
    pub chain: String,
    pub url: String,
    pub status: String,
    pub priority: i32,
}

impl Node {
    pub fn as_primitive(&self) -> ChainNode {
        ChainNode {
            chain: self.chain.clone(),
            node: self.as_primitive_node(),
        }
    }

    pub fn as_primitive_node(&self) -> primitives::node::Node {
        primitives::node::Node {
            url: self.url.clone(),
            status: NodeState::from_str(&self.status).unwrap(),
            priority: self.priority,
        }
    }
}
