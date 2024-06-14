use crate::block_explorer::{BlockExplorer, Metadata};

static TONVIEWER_NAME: &str = "TonViewer";
static TONVIEWER_BASE_URL: &str = "https://tonviewer.com";

pub struct TonViewer {
    pub meta: Metadata,
}

impl TonViewer {
    pub fn new() -> Self {
        Self {
            meta: Metadata {
                name: TONVIEWER_NAME,
                base_url: TONVIEWER_BASE_URL,
            },
        }
    }
}

impl BlockExplorer for TonViewer {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/transaction/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        Some(self.get_address_url(_token))
    }
}
