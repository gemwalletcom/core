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
                base_url: blockchair_url!("dogecoin"),
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

    pub fn new_bnb() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("bnb"),
            },
        })
    }

    pub fn new_polygon() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("polygon"),
            },
        })
    }

    pub fn new_arbitrum() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("arbitrum-one"),
            },
        })
    }

    pub fn new_optimism() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("optimism"),
            },
        })
    }

    pub fn new_avalanche() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("avalanche"),
            },
        })
    }

    pub fn new_opbnb() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("opbnb"),
            },
        })
    }

    pub fn new_fantom() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("fantom"),
            },
        })
    }

    pub fn new_gnosis() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("gnosis-chain"),
            },
        })
    }

    pub fn new_linea() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("linea"),
            },
        })
    }

    pub fn new_solana() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("solana"),
            },
        })
    }

    pub fn new_ton() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("ton"),
            },
        })
    }

    pub fn new_tron() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("tron"),
            },
        })
    }

    pub fn new_xrp() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("xrp-ledger"),
            },
        })
    }

    pub fn new_aptos() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKCHAIR_NAME,
                base_url: blockchair_url!("aptos"),
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

    fn get_validator_url(&self, _validator: &str) -> Option<String> {
        None
    }
}
