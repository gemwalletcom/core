#[typeshare(swift="Codable")]
struct Node {
    url: String,
    status: NodeStatus,
    priority: i32,
}

#[typeshare(swift="Codable")]
struct ChainNode {
    chain: String,
    node: Node
}

#[typeshare(swift="Codable")]
struct ChainNodes {
    chain: String,
    nodes: Vec<Node>
}

#[typeshare(swift="Codable")]
#[serde(rename_all = "camelCase")]
struct NodesResponse {
    version: i32,
    nodes: Vec<ChainNodes>
}

#[typeshare(swift = "Equatable, Codable, CaseIterable")]
pub enum NodeStatus {
    active,
    inactive,
}