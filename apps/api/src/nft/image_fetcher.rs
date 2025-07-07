use reqwest::Client as HttpClient;
use std::collections::HashMap;
use std::{error::Error, time::Duration};

#[derive(Clone)]
pub struct ImageFetcher {
    client: HttpClient,
}

impl ImageFetcher {
    pub fn new() -> Self {
        Self {
            client: HttpClient::builder().timeout(Duration::from_secs(10)).build().unwrap(),
        }
    }

    pub async fn fetch(&self, url: &str) -> Result<(Vec<u8>, Option<String>, HashMap<String, String>), Box<dyn Error + Send + Sync>> {
        let response = self.client.get(url).send().await?;
        let headers = response
            .headers()
            .iter()
            .filter_map(|(name, value)| value.to_str().ok().map(|v| (name.as_str().to_lowercase(), v.to_string())))
            .collect::<HashMap<String, String>>();

        let content_type = response.headers().get("content-type").and_then(|ct| ct.to_str().ok()).map(|s| s.to_string());
        let bytes = response.bytes().await.map_err(|e| format!("Failed to read response: {e}"))?.to_vec();
        Ok((bytes, content_type, headers))
    }
}
