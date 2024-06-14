use crate::block_explorer::{BlockExplorer, Metadata};
use primitives::Chain;

static MINTSCAN_NAME: &str = "MintScan";
static MINTSCAN_BASE_URL: &str = "https://mintscan.io";

pub struct MintScan {
    pub meta: Metadata,
    pub chain: &'static str,
}

impl MintScan {
    pub fn new(chain: Chain) -> Self {
        match chain {
            Chain::Cosmos => Self {
                meta: Metadata {
                    name: MINTSCAN_NAME,
                    base_url: MINTSCAN_BASE_URL,
                },
                chain: "cosmos",
            },
            Chain::Osmosis => Self {
                meta: Metadata {
                    name: MINTSCAN_NAME,
                    base_url: MINTSCAN_BASE_URL,
                },
                chain: "osmosis",
            },
            Chain::Celestia => Self {
                meta: Metadata {
                    name: MINTSCAN_NAME,
                    base_url: MINTSCAN_BASE_URL,
                },
                chain: "celestia",
            },
            Chain::Injective => Self {
                meta: Metadata {
                    name: MINTSCAN_NAME,
                    base_url: MINTSCAN_BASE_URL,
                },
                chain: "injective",
            },
            Chain::Sei => Self {
                meta: Metadata {
                    name: MINTSCAN_NAME,
                    base_url: MINTSCAN_BASE_URL,
                },
                chain: "sei",
            },
            Chain::Noble => Self {
                meta: Metadata {
                    name: MINTSCAN_NAME,
                    base_url: MINTSCAN_BASE_URL,
                },
                chain: "noble",
            },
            _ => todo!(),
        }
    }
}

impl BlockExplorer for MintScan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/{}/tx/{}", self.meta.base_url, self.chain, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/{}/account/{}", self.meta.base_url, self.chain, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        None
    }
}
