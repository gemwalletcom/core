use diesel::prelude::*;
use primitives::node::{ChainNode, Node, NodeState, NodeType};
use std::str::FromStr;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::nodes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NodeRow {
    pub id: i32,
    pub chain: String,
    pub url: String,
    pub node_type: String,
    pub status: String,
    pub priority: i32,
}

impl NodeRow {
    pub fn as_primitive(&self) -> ChainNode {
        ChainNode {
            chain: self.chain.clone(),
            node: self.as_primitive_node(),
        }
    }

    pub fn as_primitive_node(&self) -> Node {
        Node {
            url: self.url.clone(),
            node_type: NodeType::from_str(&self.node_type).unwrap_or_default(),
            status: NodeState::from_str(&self.status).unwrap_or_default(),
            priority: self.priority,
        }
    }
}
