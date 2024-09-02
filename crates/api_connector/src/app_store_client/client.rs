use reqwest::Error;

use super::models::AppStoreResponse;

pub struct AppStoreClient {
    base_url: String,
    client: reqwest::Client,
}

impl AppStoreClient {
    pub fn new() -> Self {
        AppStoreClient {
            base_url: "https://itunes.apple.com".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn search_apps(&self, term: &str, country: &str, limit: u32) -> Result<AppStoreResponse, Error> {
        let url = format!("{}/search", self.base_url);
        let query = [("term", term), ("country", country), ("entity", "software"), ("limit", &limit.to_string())];
        let response = self.client.get(&url).query(&query).send().await?.json::<AppStoreResponse>().await?;
        Ok(response)
    }
}
