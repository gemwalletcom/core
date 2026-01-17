use crate::proxy::CachedResponse;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub response: CachedResponse,
    pub expires_at: Option<Instant>,
    pub created_at: Instant,
}

impl CacheEntry {
    pub fn new(response: CachedResponse, ttl_seconds: u64) -> Self {
        let expires_at = if ttl_seconds == 0 {
            None
        } else {
            Some(Instant::now() + Duration::from_secs(ttl_seconds))
        };
        Self {
            response,
            expires_at,
            created_at: Instant::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| Instant::now() > exp)
    }

    pub fn size(&self) -> usize {
        self.response.body.len() + self.response.content_type.len() + 64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::constants::JSON_CONTENT_TYPE;
    use reqwest::StatusCode;

    #[test]
    fn test_cache_entry_with_ttl() {
        let response = CachedResponse::new(b"test".to_vec(), StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string(), 60);
        let entry = CacheEntry::new(response, 60);

        assert!(entry.expires_at.is_some());
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_without_ttl() {
        let response = CachedResponse::new(b"test".to_vec(), StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string(), 0);
        let entry = CacheEntry::new(response, 0);

        assert!(entry.expires_at.is_none());
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_size() {
        let body = b"hello world".to_vec();
        let content_type = "application/json".to_string();
        let response = CachedResponse::new(body.clone(), StatusCode::OK.as_u16(), content_type.clone(), 60);
        let entry = CacheEntry::new(response, 60);

        assert_eq!(entry.size(), body.len() + content_type.len() + 64);
    }
}
