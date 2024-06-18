use primitives::node::{ChainNodes, Node, NodeStatus, NodesResponse};
use std::{collections::HashMap, error::Error};
use storage::DatabaseClient;

#[allow(dead_code)]
pub struct Client {
    database: DatabaseClient,
}

impl Client {
    pub async fn new(database: DatabaseClient) -> Self {
        Self { database }
    }

    #[allow(dead_code)]
    pub async fn get_nodes(&mut self) -> Result<NodesResponse, Box<dyn Error>> {
        let nodes = self.database.get_nodes()?;
        let version = self.database.get_nodes_version()?;

        let mut nodes_map: HashMap<String, Vec<Node>> = HashMap::new();
        nodes.into_iter().for_each(|node| {
            nodes_map.entry(node.chain.clone()).or_default().push(Node {
                url: node.url,
                status: if node.status == *"active" {
                    NodeStatus::Active
                } else {
                    NodeStatus::Inactive
                },
                priority: node.priority,
            });
        });

        let nodes = nodes_map
            .into_iter()
            .map(|x| ChainNodes {
                chain: x.0,
                nodes: x.1,
            })
            .collect();

        let result = NodesResponse { version, nodes };

        Ok(result)
    }
}
