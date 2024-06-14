use crate::block_explorer::{BlockExplorer, Metadata};

static BLOCKCHAIR_NAME: &str = "Blockchair";

macro_rules! blockchair_url {
    ($chain:expr) => {
        concat!("https://blockchair.com/", $chain)
    };
}

pub struct Blockchair {
    pub meta: Metadata,
}

impl Blockchair {
    pub fn new_bitcoin() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("bitcoin"),
            },
        })
    }

    pub fn new_litecoin() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("litecoin"),
            },
        })
    }

    pub fn new_doge() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("doge"),
            },
        })
    }

    pub fn new_ethereum() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("ethereum"),
            },
        })
    }

    pub fn new_base() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("base"),
            },
        })
    }
}

impl BlockExplorer for Blockchair {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/transaction/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        None
    }
}
