use cacher::{CacheKey, CacherClient};
use primitives::{SupportConversation, SupportMessage, SupportMessageInput, SupportTyping};
use rocket::{State, get, post, serde::json::Json, tokio::sync::Mutex};
use std::{error::Error, future::Future};
use storage::models::DeviceRow;
use support::{ChatwootClient, ChatwootError, ChatwootSession};

use crate::{
    devices::guard::AuthenticatedDevice,
    responders::{ApiError, ApiResponse},
};

pub struct SupportApiClient {
    chatwoot: ChatwootClient,
    cacher: CacherClient,
}

impl SupportApiClient {
    pub fn new(url: String, website_token: String, cacher: CacherClient) -> Self {
        Self {
            chatwoot: ChatwootClient::new(url, website_token),
            cacher,
        }
    }

    pub async fn conversation(&self, device: &DeviceRow) -> Result<Option<SupportConversation>, Box<dyn Error + Send + Sync>> {
        self.with_session(device, |session| async move { self.chatwoot.conversation(&session).await }).await
    }

    pub async fn messages(&self, device: &DeviceRow, before: Option<i64>, after: Option<i64>) -> Result<Vec<SupportMessage>, Box<dyn Error + Send + Sync>> {
        self.with_session(device, |session| async move { self.chatwoot.messages(&session, before, after).await })
            .await
    }

    pub async fn send_message(&self, device: &DeviceRow, input: SupportMessageInput) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        let content = input.content;
        self.with_session(device, |session| {
            let content = content.clone();
            async move { self.chatwoot.send_message(&session, content).await }
        })
        .await
    }

    pub async fn set_typing(&self, device: &DeviceRow, typing: SupportTyping) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let status = typing.status;
        self.with_session(device, |session| {
            let status = status.clone();
            async move { self.chatwoot.set_typing(&session, status).await }
        })
        .await
    }

    pub async fn update_last_seen(&self, device: &DeviceRow) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.with_session(device, |session| async move { self.chatwoot.update_last_seen(&session).await }).await
    }

    async fn with_session<T, F, Fut>(&self, device: &DeviceRow, call: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        F: Fn(ChatwootSession) -> Fut,
        Fut: Future<Output = Result<T, ChatwootError>>,
    {
        let session = self.chatwoot_session(device).await?;
        match call(session).await {
            Ok(value) => Ok(value),
            Err(error) if error.is_session_error() => {
                let session = self.refresh_session(device).await?;
                Ok(call(session).await?)
            }
            Err(error) => Err(Box::new(error)),
        }
    }

    async fn chatwoot_session(&self, device: &DeviceRow) -> Result<ChatwootSession, Box<dyn Error + Send + Sync>> {
        let cache_key = CacheKey::SupportDeviceSession(&device.device_id);
        if let Some(session) = self.cacher.get_cached_optional(cache_key).await? {
            return Ok(session);
        }
        self.refresh_session(device).await
    }

    async fn refresh_session(&self, device: &DeviceRow) -> Result<ChatwootSession, Box<dyn Error + Send + Sync>> {
        let cache_key = CacheKey::SupportDeviceSession(&device.device_id);
        let session = self.chatwoot.create_session(&device.as_primitive()).await?;
        self.cacher.set_cached(cache_key, &session).await?;
        Ok(session)
    }
}

#[get("/support")]
pub async fn get_support_conversation(device: AuthenticatedDevice, client: &State<Mutex<SupportApiClient>>) -> Result<ApiResponse<Option<SupportConversation>>, ApiError> {
    Ok(client.lock().await.conversation(&device.device_row).await?.into())
}

#[get("/support/messages?<before>&<after>")]
pub async fn get_support_messages(
    device: AuthenticatedDevice,
    before: Option<i64>,
    after: Option<i64>,
    client: &State<Mutex<SupportApiClient>>,
) -> Result<ApiResponse<Vec<SupportMessage>>, ApiError> {
    Ok(client.lock().await.messages(&device.device_row, before, after).await?.into())
}

#[post("/support/messages", format = "json", data = "<input>")]
pub async fn post_support_message(
    device: AuthenticatedDevice,
    input: Json<SupportMessageInput>,
    client: &State<Mutex<SupportApiClient>>,
) -> Result<ApiResponse<SupportMessage>, ApiError> {
    Ok(client.lock().await.send_message(&device.device_row, input.into_inner()).await?.into())
}

#[post("/support/typing", format = "json", data = "<typing>")]
pub async fn post_support_typing(device: AuthenticatedDevice, typing: Json<SupportTyping>, client: &State<Mutex<SupportApiClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.lock().await.set_typing(&device.device_row, typing.into_inner()).await?.into())
}

#[post("/support/last_seen")]
pub async fn post_support_last_seen(device: AuthenticatedDevice, client: &State<Mutex<SupportApiClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.lock().await.update_last_seen(&device.device_row).await?.into())
}
