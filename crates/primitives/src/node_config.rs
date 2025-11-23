use super::chain::Chain;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub url: String,
    pub priority: NodePriority,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    Chain::all().into_iter().map(|chain| (chain.to_string(), get_nodes_for_chain(chain))).collect()
}

pub fn get_nodes_for_chain(chain: Chain) -> Vec<Node> {
    match chain {
        Chain::Bitcoin | Chain::Litecoin | Chain::BitcoinCash => vec![],
        Chain::Ethereum => vec![
            Node::new("https://ethereum.publicnode.com", NodePriority::High),
            Node::new("https://ethereum-rpc.polkachu.com", NodePriority::High),
            Node::new("https://eth.merkle.io", NodePriority::High),
        ],
        Chain::SmartChain => vec![
            Node::new("https://bsc.publicnode.com", NodePriority::High),
            Node::new("https://bsc.merkle.io", NodePriority::High),
        ],
        Chain::Solana => vec![Node::new("https://api.mainnet-beta.solana.com", NodePriority::High)],
        Chain::Polygon => vec![
            Node::new("https://polygon.llamarpc.com", NodePriority::High),
            Node::new("https://polygon-rpc.com", NodePriority::High),
        ],
        Chain::Thorchain => vec![Node::new("https://daemon.thorchain.shapeshift.com/lcd", NodePriority::High)],
        Chain::Cosmos => vec![
            Node::new("https://cosmos-rest.publicnode.com", NodePriority::High),
            Node::new("https://cosmos-api.polkachu.com", NodePriority::High),
            Node::new("https://rest.cosmos.directory/cosmoshub", NodePriority::High),
        ],
        Chain::Osmosis => vec![
            Node::new("https://osmosis-rest.publicnode.com", NodePriority::High),
            Node::new("https://osmosis-api.polkachu.com", NodePriority::High),
        ],
        Chain::Arbitrum => vec![
            Node::new("https://arb1.arbitrum.io/rpc", NodePriority::High),
            Node::new("https://arbitrum-rpc.polkachu.com", NodePriority::High),
            Node::new("https://arbitrum-one.publicnode.com", NodePriority::High),
        ],
        Chain::Ton => vec![Node::new("https://toncenter.com", NodePriority::High)],
        Chain::Tron => vec![
            Node::new("https://api.trongrid.io", NodePriority::High),
            Node::new("https://api.frankfurt.trongrid.io", NodePriority::High),
            Node::new("https://tron-rpc.publicnode.com", NodePriority::High),
        ],
        Chain::Doge => vec![],
        Chain::Zcash => vec![],
        Chain::Optimism => vec![
            Node::new("https://mainnet.optimism.io", NodePriority::High),
            Node::new("https://optimism-rpc.polkachu.com", NodePriority::High),
        ],
        Chain::Aptos => vec![
            Node::new("https://fullnode.mainnet.aptoslabs.com", NodePriority::High),
            Node::new("https://aptos-fullnode.polkachu.com", NodePriority::High),
        ],
        Chain::Base => vec![
            Node::new("https://mainnet.base.org", NodePriority::High),
            Node::new("https://base-rpc.polkachu.com", NodePriority::High),
            Node::new("https://base.merkle.io", NodePriority::High),
        ],
        Chain::AvalancheC => vec![Node::new("https://avalanche.drpc.org", NodePriority::High)],
        Chain::Sui => vec![
            Node::new("https://sui-rpc.publicnode.com", NodePriority::High),
            Node::new("https://rpc-mainnet.suiscan.xyz", NodePriority::High),
        ],
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
            Node::new("https://rpc.gnosischain.com", NodePriority::High),
        ],
        Chain::Celestia => vec![
            Node::new("https://celestia-rest.publicnode.com", NodePriority::High),
            Node::new("https://celestia-api.polkachu.com", NodePriority::High),
        ],
        Chain::Injective => vec![
            Node::new("https://injective-rest.publicnode.com", NodePriority::High),
            Node::new("https://injective-api.polkachu.com", NodePriority::High),
        ],
        Chain::Sei => vec![
            Node::new("https://rest.sei-apis.com", NodePriority::High),
            Node::new("https://api-sei.stingray.plus", NodePriority::High),
            Node::new("https://sei-api.polkachu.com", NodePriority::High),
        ],
        Chain::Manta => vec![
            Node::new("https://pacific-rpc.manta.network/http", NodePriority::High),
            Node::new("https://manta-pacific.drpc.org", NodePriority::High),
        ],
        Chain::Blast => vec![Node::new("https://blast-rpc.polkachu.com", NodePriority::High)],
        Chain::Noble => vec![
            Node::new("https://rest.cosmos.directory/noble", NodePriority::High),
            Node::new("https://noble-api.polkachu.com", NodePriority::High),
        ],
        Chain::ZkSync => vec![
            Node::new("https://zksync.drpc.org", NodePriority::High),
            Node::new("https://mainnet.era.zksync.io", NodePriority::High),
        ],
        Chain::Linea => vec![Node::new("https://linea-rpc.polkachu.com", NodePriority::High)],
        Chain::Mantle => vec![Node::new("https://rpc.mantle.xyz", NodePriority::High)],
        Chain::Celo => vec![],
        Chain::Near => vec![Node::new("https://rpc.mainnet.near.org", NodePriority::High)],
        Chain::World => vec![Node::new("https://worldchain-mainnet.gateway.tenderly.co", NodePriority::High)],
        Chain::Stellar => vec![Node::new("https://horizon.stellar.org", NodePriority::High)],
        Chain::Sonic => vec![Node::new("https://rpc.soniclabs.com", NodePriority::High)],
        Chain::Algorand => vec![Node::new("https://mainnet-api.algonode.cloud", NodePriority::High)],
        Chain::Polkadot => vec![Node::new("https://polkadot-public-sidecar.parity-chains.parity.io", NodePriority::High)],
        Chain::Plasma => vec![Node::new("https://rpc.plasma.to", NodePriority::High)],
        Chain::Cardano => vec![],
        Chain::Abstract => vec![Node::new("https://api.mainnet.abs.xyz", NodePriority::High)],
        Chain::Berachain => vec![Node::new("https://rpc.berachain.com", NodePriority::High)],
        Chain::Ink => vec![
            Node::new("https://rpc-qnd.inkonchain.com", NodePriority::High),
            Node::new("https://rpc-gel.inkonchain.com", NodePriority::High),
        ],
        Chain::Unichain => vec![
            Node::new("https://mainnet.unichain.org", NodePriority::High),
            Node::new("https://unichain-rpc.publicnode.com", NodePriority::High),
        ],
        Chain::Hyperliquid => vec![
            Node::new("https://rpc.hyperliquid.xyz/evm", NodePriority::High),
            Node::new("https://rpc.hypurrscan.io", NodePriority::High),
            Node::new("https://rpc.hyperlend.finance", NodePriority::High),
            Node::new("https://hyperliquid-json-rpc.stakely.io", NodePriority::High),
        ],
        Chain::HyperCore => vec![Node::new("https://api.hyperliquid.xyz", NodePriority::High)],
        Chain::Monad => vec![
          Node::new("https://rpc.monad.xyz", NodePriority::High),
          Node::new("https://rpc1.monad.xyz", NodePriority::Medium),
        ],
        Chain::XLayer => vec![
            Node::new("https://rpc.xlayer.tech", NodePriority::High),
            Node::new("https://xlayerrpc.okx.com", NodePriority::High),
        ],
    }
}
