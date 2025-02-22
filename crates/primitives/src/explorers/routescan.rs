use crate::block_explorer::{BlockExplorer, Metadata};

pub struct RouteScan {
    pub meta: Metadata,
}

static ROUTE_SCAN: &str = "RouteScan";

impl RouteScan {
    pub fn new_avax() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "SnowTrace",
                base_url: "https://snowtrace.io",
            },
        })
    }

    pub fn new_sonic() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: ROUTE_SCAN,
                base_url: "https://146.routescan.io",
            },
        })
    }

    pub fn new_ink() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: ROUTE_SCAN,
                base_url: "https://57073.routescan.io",
            },
        })
    }
}

impl BlockExplorer for RouteScan {
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
}
