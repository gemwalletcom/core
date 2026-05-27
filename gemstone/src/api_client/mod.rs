use crate::alien::{AlienProvider, AlienTarget};
use primitives::{ResponseResult, ScanTransaction, ScanTransactionPayload, SupportConversation, SupportMessage, SupportMessageInput, SupportTyping};
use serde::de::DeserializeOwned;
use serde_json::{from_slice, json};
use std::sync::Arc;
use url::Url;

#[derive(Debug, Clone)]
pub struct GemApiClient {
    api_url: String,
    provider: Arc<dyn AlienProvider>,
}

impl GemApiClient {
    pub fn new(api_url: String, provider: Arc<dyn AlienProvider>) -> Self {
        Self { api_url, provider }
    }

    pub async fn scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, String> {
        let url = self.url("/v1/scan/transaction")?;
        let target = AlienTarget::post_json(&url, &payload);
        let response = self.provider.request(target).await.map_err(|e| e.to_string())?;
        from_slice(&response.data).map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn support_conversation(&self) -> Result<Option<SupportConversation>, String> {
        self.request_json(AlienTarget::get(&self.url("/v1/support")?)).await
    }

    pub async fn support_messages(&self, before: Option<i64>, after: Option<i64>) -> Result<Vec<SupportMessage>, String> {
        let url = self.url_with_query(
            "/v1/support/messages",
            &[("before", before.map(|value| value.to_string())), ("after", after.map(|value| value.to_string()))],
        )?;
        self.request_json(AlienTarget::get(&url)).await
    }

    pub async fn send_support_message(&self, input: SupportMessageInput) -> Result<SupportMessage, String> {
        let url = self.url("/v1/support/messages")?;
        self.request_json(AlienTarget::post_json(&url, &input)).await
    }

    pub async fn set_support_typing(&self, typing: SupportTyping) -> Result<bool, String> {
        let url = self.url("/v1/support/typing")?;
        self.request_json(AlienTarget::post_json(&url, &typing)).await
    }

    pub async fn update_support_last_seen(&self) -> Result<bool, String> {
        let url = self.url("/v1/support/last_seen")?;
        self.request_json(AlienTarget::post_json(&url, &json!({}))).await
    }

    async fn request_json<T: DeserializeOwned>(&self, target: AlienTarget) -> Result<T, String> {
        let response = self.provider.request(target).await.map_err(|e| e.to_string())?;
        if let Some(status) = response.status
            && !(200..300).contains(&status)
        {
            return Err(format!("HTTP error: status {status}"));
        }

        match from_slice::<ResponseResult<T>>(&response.data).map_err(|e| format!("Failed to parse response: {}", e))? {
            ResponseResult::Success(value) => Ok(value),
            ResponseResult::Error(error) => Err(error.error.message),
        }
    }

    fn url(&self, path: &str) -> Result<String, String> {
        let base = self.api_url.trim_end_matches('/');
        Ok(Url::parse(&format!("{base}{path}")).map_err(|error| error.to_string())?.to_string())
    }

    fn url_with_query(&self, path: &str, query: &[(&str, Option<String>)]) -> Result<String, String> {
        let mut url = Url::parse(&self.url(path)?).map_err(|error| error.to_string())?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                if let Some(value) = value {
                    pairs.append_pair(key, value);
                }
            }
        }
        Ok(url.to_string())
    }
}
