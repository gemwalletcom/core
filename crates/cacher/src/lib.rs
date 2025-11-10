use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use redis::{AsyncCommands, Client, aio::MultiplexedConnection};

mod error;
mod keys;
pub use error::*;
pub use keys::*;

#[derive(Clone)]
pub struct CacherClient {
    connection: MultiplexedConnection,
}

impl CacherClient {
    pub async fn new(redis_url: &str) -> Self {
        let client = Client::open(redis_url).unwrap();
        let connection = client.get_multiplexed_async_connection().await.unwrap();
        Self { connection }
    }

    pub async fn set_values(&mut self, values: Vec<(String, String)>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.connection.mset::<String, String, ()>(values.as_slice()).await?;
        Ok(values.len())
    }

    pub async fn set_values_with_publish(&mut self, values: Vec<(String, String)>, ttl_seconds: i64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut pipe = redis::pipe();
        for (key, value) in &values {
            pipe.cmd("SET").arg(key).arg(value).arg("EX").arg(ttl_seconds).ignore();
            pipe.cmd("PUBLISH").arg(key).arg(value).ignore();
        }
        pipe.query_async::<()>(&mut self.connection.clone()).await?;
        Ok(values.len())
    }

    pub async fn set_value_with_ttl(&mut self, key: &str, value: String, seconds: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(self.connection.set_ex::<&str, String, ()>(key, value.clone(), seconds).await?)
    }

    pub async fn set_value<T: serde::Serialize>(&mut self, key: &str, value: &T) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(self.connection.set::<&str, String, ()>(key, serde_json::to_string(value)?).await?)
    }

    pub async fn get_value<T: serde::de::DeserializeOwned>(&mut self, key: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        let value: Option<String> = self.connection.get(key).await?;
        match value {
            Some(s) => Ok(serde_json::from_str(&s)?),
            None => Err(Box::new(CacheError::NotFound(key.to_string()))),
        }
    }

    pub async fn get_values<T, I>(&mut self, keys: Vec<String>) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        I: serde::de::DeserializeOwned,
        T: FromIterator<I>,
    {
        let result: Vec<Option<String>> = self.connection.mget(keys).await?;
        let values: T = result
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
            .iter()
            .filter_map(|x| serde_json::from_str::<I>(x).ok())
            .collect();
        Ok(values)
    }

    pub async fn get_or_set_value<T, F, Fut>(&mut self, key: &str, fetch_fn: F, ttl_seconds: Option<u64>) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        if let Ok(cached_value) = self.get_value::<T>(key).await {
            return Ok(cached_value);
        }

        let fresh_value = fetch_fn().await?;

        let serialized = serde_json::to_string(&fresh_value)?;
        if let Some(ttl) = ttl_seconds {
            self.set_value_with_ttl(key, serialized, ttl).await?;
        } else {
            self.connection.set::<&str, String, ()>(key, serialized).await?;
        }

        Ok(fresh_value)
    }

    pub async fn set_hset<T: serde::Serialize>(&mut self, hash_key: &str, field: &str, value: &T) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let serialized_value = serde_json::to_string(value)?;
        let redis_result: i64 = self.connection.hset(hash_key, field, serialized_value).await?;
        Ok(redis_result == 1)
    }

    pub async fn can_process_now(&mut self, key: &str, ttl_seconds: u64) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let last_processed: u64 = self.get_or_set_value(key, || async { Ok(now) }, Some(ttl_seconds)).await?;
        Ok(last_processed == now)
    }
}
