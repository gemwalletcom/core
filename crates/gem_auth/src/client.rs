use cacher::CacherClient;
use chrono::Utc;
use primitives::AuthNonce;
use std::error::Error;
use uuid::Uuid;

const NONCE_TTL_SECONDS: u64 = 300; // 5 minutes
const NONCE_KEY_PREFIX: &str = "auth:nonce:";

pub struct AuthClient {
    cacher: CacherClient,
}

impl AuthClient {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }

    pub async fn get_nonce(&self, device_id: &str) -> Result<AuthNonce, Box<dyn Error + Send + Sync>> {
        let auth_nonce = AuthNonce {
            nonce: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp() as u32,
        };
        let key = format!("{}{}:{}", NONCE_KEY_PREFIX, device_id, auth_nonce.nonce);
        let value = serde_json::to_string(&auth_nonce)?;
        self.cacher.set_value_with_ttl(&key, value, NONCE_TTL_SECONDS).await?;
        Ok(auth_nonce)
    }

    pub async fn get_auth_nonce(&self, device_id: &str, nonce: &str) -> Result<AuthNonce, Box<dyn Error + Send + Sync>> {
        let key = format!("{}{}:{}", NONCE_KEY_PREFIX, device_id, nonce);
        self.cacher.get_value::<AuthNonce>(&key).await
    }

    pub async fn invalidate_nonce(&self, device_id: &str, nonce: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let key = format!("{}{}:{}", NONCE_KEY_PREFIX, device_id, nonce);
        self.cacher.delete(&key).await?;
        Ok(())
    }
}
