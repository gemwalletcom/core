use super::models::Validator;
use primitives::Chain;

#[derive(Clone)]
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

    pub async fn get_assets_list(&self, chain: Chain) -> Result<Vec<String>, reqwest::Error> {
        let url = format!("{}/blockchains/{}/assets.json", self.url, chain.as_ref());
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        response.json().await
    }
}
