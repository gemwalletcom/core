use primitives::{Device, SupportConversation, SupportMessage, SupportTypingStatus};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;

use crate::{
    ChatwootConfigResponse, ChatwootContactResponse, ChatwootContactUpdate, ChatwootConversationResponse, ChatwootError, ChatwootMessageInput, ChatwootMessagesResponse,
    ChatwootSession, ChatwootTypingInput, Message, support_conversation_status, support_messages, timestamp, unread_count,
};

const PATH_CONFIG: &str = "config";
const PATH_CONTACT_SET_USER: &str = "contact/set_user";
const PATH_CONVERSATIONS: &str = "conversations";
const PATH_MESSAGES: &str = "messages";
const PATH_TOGGLE_TYPING: &str = "conversations/toggle_typing";
const PATH_UPDATE_LAST_SEEN: &str = "conversations/update_last_seen";

#[derive(Clone)]
pub struct ChatwootClient {
    client: Client,
    url: String,
    website_token: String,
}

impl ChatwootClient {
    pub fn new(url: String, website_token: String) -> Self {
        Self {
            client: Client::new(),
            url: url.trim_end_matches('/').to_string(),
            website_token,
        }
    }

    pub async fn create_session(&self, device: &Device) -> Result<ChatwootSession, ChatwootError> {
        let response: ChatwootConfigResponse = self
            .json(
                self.client
                    .post(self.widget_url(PATH_CONFIG))
                    .query(&[("website_token", self.website_token.as_str())])
                    .send()
                    .await?,
            )
            .await?;

        let update = ChatwootContactUpdate::new(device);
        let contact: ChatwootContactResponse = self
            .json(
                self.client
                    .patch(self.widget_url(PATH_CONTACT_SET_USER))
                    .query(&[("website_token", self.website_token.as_str())])
                    .headers(self.auth_headers(&response.website_channel_config.auth_token)?)
                    .json(&update)
                    .send()
                    .await?,
            )
            .await?;

        Ok(ChatwootSession {
            auth_token: contact.widget_auth_token.unwrap_or(response.website_channel_config.auth_token),
        })
    }

    pub async fn conversation(&self, session: &ChatwootSession) -> Result<Option<SupportConversation>, ChatwootError> {
        let conversation: ChatwootConversationResponse = self
            .json(
                self.client
                    .get(self.widget_url(PATH_CONVERSATIONS))
                    .query(&[("website_token", self.website_token.as_str())])
                    .headers(self.auth_headers(&session.auth_token)?)
                    .send()
                    .await?,
            )
            .await?;

        let Some(id) = conversation.id else {
            return Ok(None);
        };

        let messages = self.messages(session, None, None).await?;
        let first_message = messages.iter().find(|message| message.sender.is_user()).map(|message| message.content.clone());
        let last_message = messages.last().map(|message| message.content.clone());
        let last_activity_at = messages
            .last()
            .map(|message| message.created_at)
            .or_else(|| conversation.contact_last_seen_at.and_then(timestamp))
            .ok_or_else(|| ChatwootError::invalid_response("conversation has no activity timestamp"))?;

        Ok(Some(SupportConversation {
            id: id.to_string(),
            status: support_conversation_status(conversation.status.as_deref()),
            first_message,
            last_message,
            last_activity_at,
            unread_count: unread_count(&messages, conversation.contact_last_seen_at),
        }))
    }

    pub async fn messages(&self, session: &ChatwootSession, before: Option<i64>, after: Option<i64>) -> Result<Vec<SupportMessage>, ChatwootError> {
        let mut query = vec![("website_token", self.website_token.clone())];
        if let Some(before) = before {
            query.push(("before", before.to_string()));
        }
        if let Some(after) = after {
            query.push(("after", after.to_string()));
        }

        let response: ChatwootMessagesResponse = self
            .json(
                self.client
                    .get(self.widget_url(PATH_MESSAGES))
                    .query(&query)
                    .headers(self.auth_headers(&session.auth_token)?)
                    .send()
                    .await?,
            )
            .await?;

        Ok(support_messages(&response.payload))
    }

    pub async fn send_message(&self, session: &ChatwootSession, content: String) -> Result<SupportMessage, ChatwootError> {
        let message: Message = self
            .json(
                self.client
                    .post(self.widget_url(PATH_MESSAGES))
                    .query(&[("website_token", self.website_token.as_str())])
                    .headers(self.auth_headers(&session.auth_token)?)
                    .json(&ChatwootMessageInput::new(content))
                    .send()
                    .await?,
            )
            .await?;

        message
            .support_message()
            .ok_or_else(|| ChatwootError::invalid_response("message response is not a public text message"))
    }

    pub async fn set_typing(&self, session: &ChatwootSession, status: SupportTypingStatus) -> Result<bool, ChatwootError> {
        self.empty(
            self.client
                .post(self.widget_url(PATH_TOGGLE_TYPING))
                .query(&[("website_token", self.website_token.as_str())])
                .headers(self.auth_headers(&session.auth_token)?)
                .json(&ChatwootTypingInput::new(status))
                .send()
                .await?,
        )
        .await
    }

    pub async fn update_last_seen(&self, session: &ChatwootSession) -> Result<bool, ChatwootError> {
        self.empty(
            self.client
                .post(self.widget_url(PATH_UPDATE_LAST_SEEN))
                .query(&[("website_token", self.website_token.as_str())])
                .headers(self.auth_headers(&session.auth_token)?)
                .send()
                .await?,
        )
        .await
    }

    fn widget_url(&self, path: &str) -> String {
        format!("{}/api/v1/widget/{}", self.url, path)
    }

    fn auth_headers(&self, token: &str) -> Result<HeaderMap, ChatwootError> {
        let value = HeaderValue::from_str(token).map_err(|error| ChatwootError::invalid_response(error.to_string()))?;
        let mut headers = HeaderMap::new();
        headers.insert(HeaderName::from_static("x-auth-token"), value);
        Ok(headers)
    }

    async fn empty(&self, response: Response) -> Result<bool, ChatwootError> {
        self.check_status(response).await?;
        Ok(true)
    }

    async fn json<T: DeserializeOwned>(&self, response: Response) -> Result<T, ChatwootError> {
        let response = self.check_status(response).await?;
        Ok(response.json::<T>().await?)
    }

    async fn check_status(&self, response: Response) -> Result<Response, ChatwootError> {
        if response.status().is_success() {
            return Ok(response);
        }
        let status = response.status().as_u16();
        let message = response.text().await?;
        Err(ChatwootError::http(status, message))
    }
}
