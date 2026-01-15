use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use primitives::ConfigKey;
use serde::de::DeserializeOwned;

use crate::repositories::config_repository::ConfigRepository;
use crate::{Database, DatabaseError};

const DEFAULT_TTL_SECONDS: u64 = 60;

struct CachedValue {
    value: String,
    expires_at: Instant,
}

pub struct ConfigCacher {
    database: Database,
    cache: RwLock<HashMap<ConfigKey, CachedValue>>,
    ttl: Duration,
}

impl ConfigCacher {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            cache: RwLock::new(HashMap::new()),
            ttl: Duration::from_secs(DEFAULT_TTL_SECONDS),
        }
    }

    fn get_cached(&self, key: &ConfigKey) -> Option<String> {
        let cache = self.cache.read().ok()?;
        let cached = cache.get(key)?;
        if cached.expires_at > Instant::now() {
            Some(cached.value.clone())
        } else {
            None
        }
    }

    fn set_cached(&self, key: ConfigKey, value: String) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(
                key,
                CachedValue {
                    value,
                    expires_at: Instant::now() + self.ttl,
                },
            );
        }
    }

    pub fn get(&self, key: ConfigKey) -> Result<String, DatabaseError> {
        if let Some(value) = self.get_cached(&key) {
            return Ok(value);
        }
        let value = self
            .database
            .client()
            .map_err(|e| DatabaseError::Error(e.to_string()))?
            .get_config(key.clone())?;
        self.set_cached(key, value.clone());
        Ok(value)
    }

    pub fn get_i32(&self, key: ConfigKey) -> Result<i32, DatabaseError> {
        Ok(self.get(key)?.parse()?)
    }

    pub fn get_i64(&self, key: ConfigKey) -> Result<i64, DatabaseError> {
        Ok(self.get(key)?.parse()?)
    }

    pub fn get_f64(&self, key: ConfigKey) -> Result<f64, DatabaseError> {
        Ok(self.get(key)?.parse()?)
    }

    pub fn get_bool(&self, key: ConfigKey) -> Result<bool, DatabaseError> {
        Ok(self.get(key)?.parse()?)
    }

    pub fn get_duration(&self, key: ConfigKey) -> Result<Duration, DatabaseError> {
        let value = self.get(key)?;
        primitives::parse_duration(&value).ok_or_else(|| DatabaseError::Error(format!("Failed to parse duration: {}", value)))
    }

    pub fn get_vec_string(&self, key: ConfigKey) -> Result<Vec<String>, DatabaseError> {
        self.get_vec(key)
    }

    pub fn get_vec<T: DeserializeOwned>(&self, key: ConfigKey) -> Result<Vec<T>, DatabaseError> {
        Ok(serde_json::from_str(&self.get(key)?)?)
    }
}
