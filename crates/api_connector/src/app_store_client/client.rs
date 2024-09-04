use super::models::{App, AppStoreError, AppStoreResponse};
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

    pub async fn lookup(&self, app_id: u64, country: &str) -> Result<App, AppStoreError> {
        let url = format!("{}/lookup", self.base_url);
        let query = [("id", &app_id.to_string()), ("country", &country.to_string())];

        let response = self.client.get(&url).query(&query).send().await?.json::<AppStoreResponse>().await?;
        match response.results.first() {
            Some(app) => Ok(app.clone()),
            None => Err(AppStoreError::AppNotFound),
        }
    }

    pub async fn search_apps(&self, term: &str, country: &str, limit: u32) -> Result<AppStoreResponse, AppStoreError> {
        let url = format!("{}/search", self.base_url);
        let query = [("term", term), ("country", country), ("entity", "software"), ("limit", &limit.to_string())];
        let response = self.client.get(&url).query(&query).send().await?.json::<AppStoreResponse>().await?;
        Ok(response)
    }
}

impl Default for AppStoreClient {
    fn default() -> Self {
        Self::new()
    }
}
