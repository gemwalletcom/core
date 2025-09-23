use super::models::Validator;
use primitives::chain::Chain;

pub struct StaticAssetsClient {
    url: String,
    client: reqwest::Client,
}

impl StaticAssetsClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_validators(&self, chain: Chain) -> Result<Vec<Validator>, reqwest::Error> {
        let url = format!("{}/blockchains/{chain}/validators.json", self.url);
        self.client.get(&url).send().await?.json().await
    }
}
