use crate::block_explorer::{BlockExplorer, Metadata};
use crate::chain_evm::EVMChain;

pub struct EtherScan {
    pub meta: Metadata,
}

impl EtherScan {
    pub fn new(chain: EVMChain) -> Box<Self> {
        Box::new(match chain {
            EVMChain::Ethereum => Self {
                meta: Metadata {
                    name: "Etherscan",
                    base_url: "https://etherscan.io",
                },
            },
            EVMChain::SmartChain => Self {
                meta: Metadata {
                    name: "BscScan",
                    base_url: "https://bscscan.com",
                },
            },
            EVMChain::Polygon => Self {
                meta: Metadata {
                    name: "PolygonScan",
                    base_url: "https://polygonscan.com",
                },
            },
            EVMChain::Arbitrum => Self {
                meta: Metadata {
                    name: "ArbiScan",
                    base_url: "https://arbiscan.io",
                },
            },
            EVMChain::Optimism => Self {
                meta: Metadata {
                    name: "Etherscan",
                    base_url: "https://optimistic.etherscan.io",
                },
            },
            EVMChain::Base => Self {
                meta: Metadata {
                    name: "BaseScan",
                    base_url: "https://basescan.org",
                },
            },
            EVMChain::AvalancheC => Self {
                meta: Metadata {
                    name: "SnowTrace",
                    base_url: "https://snowtrace.io",
                },
            },
            EVMChain::OpBNB => Self {
                meta: Metadata {
                    name: "opBNBScan",
                    base_url: "https://opbnb.bscscan.com",
                },
            },
            EVMChain::Fantom => Self {
                meta: Metadata {
                    name: "FTMScan",
                    base_url: "https://ftmscan.com",
                },
            },
            EVMChain::Gnosis => Self {
                meta: Metadata {
                    name: "GnosisScan",
                    base_url: "https://gnosisscan.io",
                },
            },
            EVMChain::Manta => Self {
                meta: Metadata {
                    name: "Socialscan",
                    base_url: "https://manta.socialscan.io",
                },
            },
            EVMChain::Blast => Self {
                meta: Metadata {
                    name: "BlastScan",
                    base_url: "https://blastscan.io",
                },
            },
            EVMChain::Linea => Self {
                meta: Metadata {
                    name: "LineaScan",
                    base_url: "https://lineascan.build",
                },
            },
            EVMChain::ZkSync => Self {
                meta: Metadata {
                    name: "zkSync Era Explorer",
                    base_url: "https://era.zksync.network",
                },
            },
            EVMChain::Celo => Self {
                meta: Metadata {
                    name: "CeloScan",
                    base_url: "https://celoscan.io",
                },
            },
            EVMChain::Mantle => Self {
                meta: Metadata {
                    name: "MantleScan",
                    base_url: "https://mantlescan.xyz",
                },
            },
            EVMChain::World => Self {
                meta: Metadata {
                    name: "WorldScan",
                    base_url: "https://worldscan.org",
                },
            },
            EVMChain::Sonic => Self {
                meta: Metadata {
                    name: "SonicScan",
                    base_url: "https://sonicscan.org",
                },
            },
            EVMChain::Abstract => Self {
                meta: Metadata {
                    name: "Abscan",
                    base_url: "https://abscan.org",
                },
            },
            EVMChain::Berachain => Self {
                meta: Metadata {
                    name: "Berachain",
                    base_url: "https://berascan.com",
                },
            },
            EVMChain::Ink => todo!(),
            EVMChain::Berachain => Self {
                meta: Metadata {
                    name: "Uniscan",
                    base_url: "https://uniscan.xyz",
                },
            },
        })
    }
}

impl BlockExplorer for EtherScan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(format!("{}/token/{}", self.meta.base_url, token))
    }
    fn get_validator_url(&self, _validator: &str) -> Option<String> {
        None
    }
}
