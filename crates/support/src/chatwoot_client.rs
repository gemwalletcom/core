use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::model::MessageType;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatwootMessagesListResponse {
    payload: Vec<ChatwootConversationMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootConversationMessage {
    pub id: i64,
    pub content: Option<String>,
    pub message_type: MessageType,
}

pub struct ChatwootApiClient {
    url: String,
    api_key: String,
    client: reqwest::Client,
}

impl ChatwootApiClient {
    pub fn new(url: String, api_key: String) -> Self {
        Self {
            url,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_message(&self, account_id: i64, conversation_id: i64, content: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v1/accounts/{}/conversations/{}/messages", self.url, account_id, conversation_id);
        let body = serde_json::json!({
            "content": content,
            "message_type": "outgoing",
        });
        self.client
            .post(&url)
            .header("api_access_token", &self.api_key)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn toggle_conversation_status(&self, account_id: i64, conversation_id: i64, status: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v1/accounts/{}/conversations/{}/toggle_status", self.url, account_id, conversation_id);
        let body = serde_json::json!({ "status": status });
        self.client
            .post(&url)
            .header("api_access_token", &self.api_key)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn get_conversation_messages(&self, account_id: i64, conversation_id: i64) -> Result<Vec<ChatwootConversationMessage>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v1/accounts/{}/conversations/{}/messages", self.url, account_id, conversation_id);
        let response = self
            .client
            .get(&url)
            .header("api_access_token", &self.api_key)
            .send()
            .await?
            .error_for_status()?
            .json::<ChatwootMessagesListResponse>()
            .await?;
        Ok(response.payload)
    }
}
