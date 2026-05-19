mod api;
mod response;

use self::api::MayanApi;
use crate::responders::ApiError;
use reqwest::{Client, Url, header::CONTENT_TYPE};
use rocket::http::ContentType;
use std::path::Path;

pub(super) use response::MayanProxyResponse;

#[derive(Debug)]
pub struct MayanProxyClient {
    client: Client,
    api_key: String,
}

impl MayanProxyClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    pub async fn get(&self, api: &str, version: &str, path: &Path, query: Option<&str>) -> Result<MayanProxyResponse, ApiError> {
        let url = self.upstream_url(MayanApi::from_path(api)?, version, path, query)?;
        let response = self.client.get(url).send().await.map_err(ApiError::internal_server_error)?;
        MayanProxyResponse::from_response(response).await
    }

    pub async fn post(&self, api: &str, version: &str, path: &Path, query: Option<&str>, body: Vec<u8>) -> Result<MayanProxyResponse, ApiError> {
        let url = self.upstream_url(MayanApi::from_path(api)?, version, path, query)?;
        let response = self
            .client
            .post(url)
            .header(CONTENT_TYPE, ContentType::JSON.to_string())
            .body(body)
            .send()
            .await
            .map_err(ApiError::internal_server_error)?;
        MayanProxyResponse::from_response(response).await
    }

    fn upstream_url(&self, api: MayanApi, version: &str, path: &Path, query: Option<&str>) -> Result<Url, ApiError> {
        let mut url = api.url(version, path, query)?;
        if api == MayanApi::Price && !self.api_key.is_empty() {
            url.query_pairs_mut().append_pair("apiKey", &self.api_key);
        }
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_url_injects_api_key() {
        let client = MayanProxyClient::new("secret".to_string());
        let url = client
            .upstream_url(MayanApi::Price, "v3", Path::new("get-swap/sui"), Some("fromToken=0x2%3A%3Asui"))
            .unwrap();

        assert_eq!(url.as_str(), "https://price-api.mayan.finance/v3/get-swap/sui?fromToken=0x2%3A%3Asui&apiKey=secret");
    }

    #[test]
    fn test_price_url_allows_empty_api_key_for_local_proxy() {
        let client = MayanProxyClient::new(String::new());
        let url = client.upstream_url(MayanApi::Price, "v3", Path::new("quote"), None).unwrap();

        assert_eq!(url.as_str(), "https://price-api.mayan.finance/v3/quote");
    }

    #[test]
    fn test_explorer_url_does_not_inject_api_key() {
        let client = MayanProxyClient::new("secret".to_string());
        let url = client.upstream_url(MayanApi::Explorer, "v3", Path::new("swap/trx/0xabc"), None).unwrap();

        assert_eq!(url.as_str(), "https://explorer-api.mayan.finance/v3/swap/trx/0xabc");
    }

    #[test]
    fn test_price_url_accepts_configured_version() {
        let client = MayanProxyClient::new("secret".to_string());
        let url = client.upstream_url(MayanApi::Price, "v4", Path::new("quote"), None).unwrap();

        assert_eq!(url.as_str(), "https://price-api.mayan.finance/v4/quote?apiKey=secret");
    }

    #[test]
    fn test_price_url_appends_api_key_without_filtering_query() {
        let client = MayanProxyClient::new("secret".to_string());
        let url = client.upstream_url(MayanApi::Price, "v3", Path::new("quote"), Some("apiKey=client-key")).unwrap();

        assert_eq!(url.as_str(), "https://price-api.mayan.finance/v3/quote?apiKey=client-key&apiKey=secret");
    }
}
