use std::error::Error;

use crate::models::{Message, Request, Response};

pub struct ClaudeClient {
    url: String,
    api_key: String,
    client: reqwest::Client,
}

impl ClaudeClient {
    pub fn new(url: String, api_key: String) -> Self {
        Self {
            url,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn generate_response(
        &self,
        system_prompt: &str,
        messages: Vec<Message>,
        model: &str,
        version: &str,
        max_tokens: u32,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v1/messages", self.url);
        let request = Request {
            model: model.to_string(),
            max_tokens,
            system: system_prompt.to_string(),
            messages,
        };
        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", version)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?;
        let text = response.content.into_iter().filter_map(|block| block.text).collect::<Vec<_>>().join("");
        Ok(text)
    }
}
