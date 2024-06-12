use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable")]
pub struct Node {
    pub url: String,
    pub status: NodeStatus,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable")]
pub struct ChainNode {
    pub chain: String,
    pub node: Node,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable")]
pub struct ChainNodes {
    pub chain: String,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable")]
#[serde(rename_all = "camelCase")]
pub struct NodesResponse {
    pub version: i32,
    pub nodes: Vec<ChainNodes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Active,
    Inactive,
}
