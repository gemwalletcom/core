use primitives::node_config::{Node, NodePriority};

// Sources:
// https://chainlist.org

#[uniffi::remote(Record)]
pub struct Node {
    pub url: String,
    pub priority: NodePriority,
}

#[uniffi::remote(Enum)]
pub enum NodePriority {
    High,
    Medium,
    Low,
    Inactive,
}
