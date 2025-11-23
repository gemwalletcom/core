use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub access_token: String,
}

#[derive(Debug, Clone)]
pub struct CachedToken {
    pub access_token: String,
    pub expires_at: SystemTime,
}

impl CachedToken {
    pub fn new(access_token: String, ttl_seconds: u64) -> Self {
        Self {
            access_token,
            expires_at: SystemTime::now() + Duration::from_secs(ttl_seconds),
        }
    }

    pub fn is_valid(&self) -> bool {
        SystemTime::now() < self.expires_at
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWidgetUrlRequest {
    #[serde(rename = "widgetParams")]
    pub params: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWidgetUrlResponse {
    pub widget_url: String,
}
