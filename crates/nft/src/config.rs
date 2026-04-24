#[derive(Debug, Clone)]
pub struct NFTProviderConfig {
    pub opensea_key: String,
    pub magiceden_key: String,
    pub ton_url: String,
}

impl NFTProviderConfig {
    pub fn new(opensea_key: String, magiceden_key: String, ton_url: String) -> Self {
        Self {
            opensea_key,
            magiceden_key,
            ton_url,
        }
    }
}
