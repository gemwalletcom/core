use primitives::{Chain, NodeType};

#[derive(Clone)]
pub struct ProviderConfig {
    pub chain: Chain,
    pub url: String,
    pub archive_url: String,
    pub ankr_key: String,
    pub trongrid_key: String,
}

impl ProviderConfig {
    pub fn new(chain: Chain, url: &str, archive_url: &str, ankr_key: &str, trongrid_key: &str) -> Self {
        Self {
            chain,
            url: url.to_string(),
            archive_url: archive_url.to_string(),
            ankr_key: ankr_key.to_string(),
            trongrid_key: trongrid_key.to_string(),
        }
    }

    pub fn node_type(&self) -> NodeType {
        if self.archive_url.is_empty() || self.archive_url == self.url {
            NodeType::Default
        } else {
            NodeType::Archive
        }
    }

    pub fn ankr_url(&self) -> String {
        format!("https://rpc.ankr.com/multichain/{}", self.ankr_key)
    }

    pub fn with_url(&self, url: &str) -> Self {
        Self {
            url: url.to_string(),
            ..self.clone()
        }
    }
}
