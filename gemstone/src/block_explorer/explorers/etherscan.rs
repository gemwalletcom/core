use crate::block_explorer::{BlockExplorer, Metadata};
use primitives::Chain;

pub struct EtherScan {
    pub meta: Metadata,
    pub account_path: &'static str,
    pub token_path: &'static str,
}

impl EtherScan {
    pub fn new_evm(chain: primitives::Chain) -> Self {
        let account_path = "address";
        let token_path = "token";
        match chain {
            Chain::Ethereum => Self {
                meta: Metadata {
                    name: "EtherScan",
                    base_url: "https://etherscan.io",
                },
                account_path,
                token_path,
            },
            Chain::SmartChain => Self {
                meta: Metadata {
                    name: "BscScan",
                    base_url: "https://bscscan.com",
                },
                account_path,
                token_path,
            },
            Chain::Polygon => Self {
                meta: Metadata {
                    name: "PolygonScan",
                    base_url: "https://polygonscan.com",
                },
                account_path,
                token_path,
            },
            Chain::Arbitrum => Self {
                meta: Metadata {
                    name: "EtherScan",
                    base_url: "https://arbiscan.io",
                },
                account_path,
                token_path,
            },
            Chain::Optimism => Self {
                meta: Metadata {
                    name: "EtherScan",
                    base_url: "https://optimistic.etherscan.io",
                },
                account_path,
                token_path,
            },
            Chain::Base => Self {
                meta: Metadata {
                    name: "BaseScan",
                    base_url: "https://basescan.org",
                },
                account_path,
                token_path,
            },
            Chain::AvalancheC => Self {
                meta: Metadata {
                    name: "SnowTrace",
                    base_url: "https://snowtrace.io",
                },
                account_path,
                token_path,
            },
            Chain::OpBNB => Self {
                meta: Metadata {
                    name: "OpBNBScan",
                    base_url: "https://opbnb.bscscan.com",
                },
                account_path,
                token_path,
            },
            Chain::Fantom => Self {
                meta: Metadata {
                    name: "FTMScan",
                    base_url: "https://ftmscan.com",
                },
                account_path,
                token_path,
            },
            Chain::Gnosis => Self {
                meta: Metadata {
                    name: "GnosisScan",
                    base_url: "https://gnosisscan.io",
                },
                account_path,
                token_path,
            },
            Chain::Manta => Self {
                meta: Metadata {
                    name: "Pacific Explorer",
                    base_url: "https://pacific-explorer.manta.network",
                },
                account_path,
                token_path,
            },
            Chain::Blast => Self {
                meta: Metadata {
                    name: "BlastScan",
                    base_url: "https://blastscan.io",
                },
                account_path,
                token_path,
            },
            Chain::Linea => Self {
                meta: Metadata {
                    name: "LineaScan",
                    base_url: "https://lineascan.build",
                },
                account_path,
                token_path,
            },
            Chain::ZkSync => Self {
                meta: Metadata {
                    name: "zkSync Era Explorer",
                    base_url: "https://era.zksync.network",
                },
                account_path,
                token_path,
            },
            Chain::Celo => Self {
                meta: Metadata {
                    name: "CeloScan",
                    base_url: "https://celoscan.io",
                },
                account_path,
                token_path,
            },
            Chain::Mantle => Self {
                meta: Metadata {
                    name: "MantleScan",
                    base_url: "https://mantlescan.xyz/",
                },
                account_path,
                token_path,
            },
            _ => todo!(),
        }
    }

    pub fn solana() -> Self {
        Self {
            meta: Metadata {
                name: "Solscan",
                base_url: "https://solscan.io",
            },
            account_path: "account",
            token_path: "token",
        }
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
        format!("{}/{}/{}", self.meta.base_url, self.account_path, address)
    }
    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(format!(
            "{}/{}/{}",
            self.meta.base_url, self.token_path, token
        ))
    }
}
