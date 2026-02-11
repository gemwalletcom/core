use std::error::Error;
use std::time::Duration;

use cacher::{CacheKey, CacherClient};
use chrono::Utc;
use primitives::{AuthNonce, DeviceToken};
use uuid::Uuid;

use crate::jwt;

pub struct AuthClient {
    cacher: CacherClient,
}

impl AuthClient {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }

    pub fn create_device_token(&self, device_id: &str, secret: &str, expiry: Duration) -> Result<DeviceToken, Box<dyn Error + Send + Sync>> {
        let (token, expires_at) = jwt::create_device_token(device_id, secret, expiry)?;
        Ok(DeviceToken { token, expires_at })
    }

    pub async fn get_nonce(&self, device_id: &str) -> Result<AuthNonce, Box<dyn Error + Send + Sync>> {
        let auth_nonce = AuthNonce {
            nonce: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp() as u32,
        };
        let cache_key = CacheKey::AuthNonce(device_id, &auth_nonce.nonce);
        let value = serde_json::to_string(&auth_nonce)?;
        self.cacher.set_value_with_ttl(&cache_key.key(), value, cache_key.ttl()).await?;
        Ok(auth_nonce)
    }

    pub async fn get_auth_nonce(&self, device_id: &str, nonce: &str) -> Result<AuthNonce, Box<dyn Error + Send + Sync>> {
        self.cacher.get_value::<AuthNonce>(&CacheKey::AuthNonce(device_id, nonce).key()).await
    }

    pub async fn invalidate_nonce(&self, device_id: &str, nonce: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.delete(&CacheKey::AuthNonce(device_id, nonce).key()).await?;
        Ok(())
    }
}
