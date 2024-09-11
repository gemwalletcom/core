use reqwest::Error;

use super::models::{AppInformation, AppSearch, Data, Results, Review};

#[derive(Debug, Clone)]
pub struct GooglePlayClient {
    base_url: String,
    client: reqwest::Client,
}

impl GooglePlayClient {
    pub fn new(url: String) -> Self {
        GooglePlayClient {
            base_url: url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn lookup(&self, app_id: String, country: &str, language: &str) -> Result<AppInformation, Error> {
        let url = format!("{}/api/apps/{}", self.base_url, app_id);
        let query = [("country", &country.to_string()), ("lang", &language.to_string())];

        self.client.get(&url).query(&query).send().await?.json::<AppInformation>().await
    }

    pub async fn search_apps(&self, term: &str, country: &str, language: &str, limit: u32) -> Result<Vec<AppSearch>, Error> {
        let url = format!("{}/api/apps/", self.base_url);
        let query = [("q", term), ("country", country), ("language", language), ("num", &limit.to_string())];
        self.client
            .get(&url)
            .query(&query)
            .send()
            .await?
            .json::<Results<Vec<AppSearch>>>()
            .await
            .map(|r| r.results)
    }

    pub async fn reviews(&self, app_id: &str, country: &str, language: &str) -> Result<Vec<Review>, Error> {
        let url = format!("{}/api/apps/{}/reviews/", self.base_url, app_id);
        let query = [("country", &country.to_string()), ("lang", &language.to_string())];
        self.client
            .get(&url)
            .query(&query)
            .send()
            .await?
            .json::<Results<Data<Vec<Review>>>>()
            .await
            .map(|r| r.results.data)
    }
}
