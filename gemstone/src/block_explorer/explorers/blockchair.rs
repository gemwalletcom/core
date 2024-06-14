use crate::block_explorer::{BlockExplorer, Metadata};
use primitives::Chain;

static BLOCKCHAIR_NAME: &str = "Blockchair";
static BLOCKCHAIR_BASE_URL: &str = "https://blockchair.com";

pub struct Blockchair {
    pub meta: Metadata,
    pub chain: &'static str,
}

impl Blockchair {
    pub fn new(chain: Chain) -> Self {
        match chain {
            Chain::Bitcoin => Self {
                meta: Metadata {
                    name: BLOCKCHAIR_NAME,
                    base_url: BLOCKCHAIR_BASE_URL,
                },
                chain: "bitcoin",
            },
            Chain::Litecoin => Self {
                meta: Metadata {
                    name: BLOCKCHAIR_NAME,
                    base_url: BLOCKCHAIR_BASE_URL,
                },
                chain: "litecoin",
            },
            Chain::Doge => Self {
                meta: Metadata {
                    name: BLOCKCHAIR_NAME,
                    base_url: BLOCKCHAIR_BASE_URL,
                },
                chain: "dogecoin",
            },
            _ => todo!(),
        }
    }
}

impl BlockExplorer for Blockchair {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/{}/transaction/{}", self.meta.base_url, self.chain, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/{}/address/{}", self.meta.base_url, self.chain, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        None
    }
}
