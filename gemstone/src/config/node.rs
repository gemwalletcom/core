use std::collections::HashMap;

use primitives::Chain;

// Sources:
// https://chainlist.org

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct Node {
    pub url: String,
    pub priority: NodePriority,
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum NodePriority {
    High = 10,
    Medium = 5,
    Low = 1,
    Inactive = -1,
}

impl Node {
    pub fn new(url: &str, priority: NodePriority) -> Self {
        Node {
            url: url.to_string(),
            priority,
        }
    }
}

pub fn get_nodes() -> HashMap<String, Vec<Node>> {
    Chain::all()
        .into_iter()
        .map(|chain| (chain.to_string(), get_nodes_for_chain(chain)))
        .collect()
}

pub fn get_nodes_for_chain(chain: Chain) -> Vec<Node> {
    match chain {
        Chain::Bitcoin | Chain::Litecoin => vec![],
        Chain::Ethereum => vec![
            Node::new("https://eth.llamarpc.com", NodePriority::High),
            Node::new("https://ethereum.publicnode.com", NodePriority::High),
            Node::new("https://rpc.ankr.com/eth", NodePriority::High),
            Node::new("https://eth.drpc.org", NodePriority::High),
        ],
        Chain::SmartChain => vec![
            Node::new("https://binance.llamarpc.com", NodePriority::High),
            Node::new("https://bsc.publicnode.com", NodePriority::High),
        ],
        Chain::Solana => vec![Node::new("https://api.mainnet-beta.solana.com", NodePriority::High)],
        Chain::Polygon => vec![
            Node::new("https://polygon.llamarpc.com", NodePriority::High),
            Node::new("https://polygon-rpc.com", NodePriority::High),
        ],
        Chain::Thorchain => vec![],
        Chain::Cosmos => vec![
            Node::new("https://cosmos-rest.publicnode.com", NodePriority::High),
            Node::new("https://rest.cosmos.directory/cosmoshub", NodePriority::High),
        ],
        Chain::Osmosis => vec![
            Node::new("https://osmosis-rest.publicnode.com", NodePriority::High),
            Node::new("https://lcd-osmosis.blockapsis.com", NodePriority::High),
        ],
        Chain::Arbitrum => vec![
            Node::new("https://arb1.arbitrum.io/rpc", NodePriority::High),
            Node::new("https://arbitrum.drpc.org", NodePriority::High),
            Node::new("https://arbitrum-one.publicnode.com", NodePriority::High),
        ],
        Chain::Ton => vec![Node::new("https://toncenter.com", NodePriority::High)],
        Chain::Tron => vec![
            Node::new("https://api.trongrid.io", NodePriority::High),
            Node::new("https://api.frankfurt.trongrid.io", NodePriority::High),
        ],
        Chain::Doge => vec![],
        Chain::Optimism => vec![
            Node::new("https://mainnet.optimism.io", NodePriority::High),
            Node::new("https://rpc.ankr.com/optimism", NodePriority::High),
        ],
        Chain::Aptos => vec![Node::new("https://fullnode.mainnet.aptoslabs.com", NodePriority::High)],
        Chain::Base => vec![
            Node::new("https://mainnet.base.org", NodePriority::High),
            Node::new("https://rpc.ankr.com/base", NodePriority::High),
        ],
        Chain::AvalancheC => vec![
            Node::new("https://avalanche.drpc.org", NodePriority::High),
            Node::new("https://rpc.ankr.com/avalanche", NodePriority::High),
        ],
        Chain::Sui => vec![Node::new("https://sui-rpc.publicnode.com", NodePriority::High)],
        Chain::Xrp => vec![
            Node::new("https://s1.ripple.com:51234", NodePriority::High),
            Node::new("https://s2.ripple.com:51234", NodePriority::High),
            Node::new("https://xrplcluster.com", NodePriority::High),
        ],
        Chain::OpBNB => vec![
            Node::new("https://opbnb.drpc.org", NodePriority::High),
            Node::new("https://opbnb-mainnet-rpc.bnbchain.org", NodePriority::High),
        ],
        Chain::Fantom => vec![
            Node::new("https://fantom.drpc.org", NodePriority::High),
            Node::new("https://rpc.fantom.network", NodePriority::High),
        ],
        Chain::Gnosis => vec![
            Node::new("https://gnosis.drpc.org", NodePriority::High),
            Node::new("https://rpc.ankr.com/gnosis", NodePriority::High),
        ],
        Chain::Celestia => vec![Node::new("https://celestia-rest.publicnode.com", NodePriority::High)],
        Chain::Injective => vec![Node::new("https://injective-rest.publicnode.com", NodePriority::High)],
        Chain::Sei => vec![
            Node::new("https://rest.sei-apis.com", NodePriority::High),
            Node::new("https://api-sei.stingray.plus", NodePriority::High),
        ],
        Chain::Manta => vec![
            Node::new("https://pacific-rpc.manta.network/http", NodePriority::High),
            Node::new("https://manta-pacific.drpc.org", NodePriority::High),
        ],
        Chain::Blast => vec![Node::new("https://rpc.blastblockchain.com", NodePriority::High)],
        Chain::Noble => vec![Node::new("https://rest.cosmos.directory/noble", NodePriority::High)],
        Chain::ZkSync => vec![
            Node::new("https://zksync.drpc.org", NodePriority::High),
            Node::new("https://mainnet.era.zksync.io", NodePriority::High),
        ],
        Chain::Linea => vec![Node::new("https://linea.drpc.org", NodePriority::High)],
        Chain::Mantle => vec![Node::new("https://rpc.ankr.com/mantle", NodePriority::High)],
        Chain::Celo => vec![Node::new("https://rpc.ankr.com/celo", NodePriority::High)],
        Chain::Near => vec![Node::new("https://rpc.mainnet.near.org", NodePriority::High)],
    }
}
