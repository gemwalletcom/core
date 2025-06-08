use primitives::Chain;

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub chain: Chain,
    pub url: String,
    pub alchemy_key: String,
}

impl ProviderConfig {
    pub fn new(chain: Chain, url: &str, alchemy_key: &str) -> Self {
        Self {
            chain,
            url: url.to_string(),
            alchemy_key: alchemy_key.to_string(),
        }
    }

    pub fn with_url(&self, url: &str) -> Self {
        Self {
            url: url.to_string(),
            ..self.clone()
        }
    }
}
