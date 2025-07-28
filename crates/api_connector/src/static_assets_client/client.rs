use super::models::Validator;
use primitives::chain::Chain;
use std::error::Error;

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

    pub async fn get_validators(&self, chain: Chain) -> Result<Vec<Validator>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/blockchains/{chain}/validators.json", self.url);
        let response = self.client.get(&url).send().await?;
        Ok(response.json().await?)
    }
}
