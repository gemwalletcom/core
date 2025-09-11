#[derive(Debug, Clone)]
pub struct NFTProviderConfig {
    pub opensea_key: String,
    pub magiceden_key: String,
}

impl NFTProviderConfig {
    pub fn new(opensea_key: String, magiceden_key: String) -> Self {
        Self { opensea_key, magiceden_key }
    }
}
