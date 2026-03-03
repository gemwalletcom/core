use cacher::{CacheKey, CacherClient};
use crate::responders::ApiError;
use primitives::SwapProvider;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::{State, post};
use swapper::near_intents::base_url;

pub struct NearIntentsProxyClient {
    client: reqwest::Client,
    cacher: CacherClient,
}

impl NearIntentsProxyClient {
    pub fn new(cacher: CacherClient) -> Self {
        Self {
            client: reqwest::Client::new(),
            cacher,
        }
    }

    pub async fn quote(&self, body: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/v0/quote/forward", base_url());
        let response = self.client.post(&url).json(&body).send().await?.json::<serde_json::Value>().await?;

        if let Some(address) = response.pointer("/quote/depositAddress").and_then(|v| v.as_str())
            && !address.is_empty()
        {
            let _ = self.cacher.add_to_set_cached(CacheKey::SwapVaultAddresses(SwapProvider::NearIntents.as_ref()), &[address.to_string()]).await;
        }

        Ok(response)
    }
}

#[post("/swaps/near_intents/quote", data = "<body>")]
pub async fn post_quote(body: Json<serde_json::Value>, client: &State<Mutex<NearIntentsProxyClient>>) -> Result<Json<serde_json::Value>, ApiError> {
    let response = client.lock().await.quote(body.0).await?;
    Ok(Json(response))
}
