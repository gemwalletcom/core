use std::error::Error;

use redis::{AsyncCommands, Client};
use std::time::{SystemTime, UNIX_EPOCH};

mod error;
mod keys;
pub use error::*;
pub use keys::*;

#[derive(Debug, Clone)]
pub struct CacherClient {
    client: Client,
}

impl CacherClient {
    pub fn new(redis_url: &str) -> Self {
        let client = redis::Client::open(redis_url).unwrap();
        Self { client }
    }

    pub async fn set_values(&mut self, values: Vec<(String, String)>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.client
            .get_multiplexed_async_connection()
            .await?
            .mset::<String, String, ()>(values.as_slice())
            .await?;
        // redis always returns "OK" instead of usize for the number of inserts
        Ok(values.len())
    }

    pub async fn set_values_with_publish(&mut self, values: Vec<(String, String)>, ttl_seconds: i64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        let mut pipe = redis::pipe();
        for (key, value) in &values {
            pipe.cmd("SET").arg(key).arg(value).arg("EX").arg(ttl_seconds).ignore();
            pipe.cmd("PUBLISH").arg(key).arg(value).ignore();
        }
        pipe.query_async::<()>(&mut connection).await?;
        Ok(values.len())
    }

    pub async fn set_value_with_ttl(&mut self, key: &str, value: String, seconds: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        Ok(connection.set_ex::<&str, String, ()>(key, value.clone(), seconds).await?)
    }

    pub async fn set_value<T: serde::Serialize>(&mut self, key: &str, value: &T) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        Ok(connection.set::<&str, String, ()>(key, serde_json::to_string(value)?).await?)
    }

    pub async fn get_value<T: serde::de::DeserializeOwned>(&mut self, key: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        let value: Option<String> = connection.get(key).await?;
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
        let result: Vec<Option<String>> = self.client.get_multiplexed_async_connection().await?.mget(keys).await?;
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
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        let cached: Option<String> = connection.get(key).await?;

        if let Some(value) = cached {
            return Ok(serde_json::from_str(&value)?);
        }

        let fresh_value = fetch_fn().await?;

        let serialized = serde_json::to_string(&fresh_value)?;
        if let Some(ttl) = ttl_seconds {
            self.set_value_with_ttl(key, serialized, ttl).await?;
        } else {
            connection.set::<&str, String, ()>(key, serialized).await?;
        }

        Ok(fresh_value)
    }

    pub async fn set_hset<T: serde::Serialize>(&mut self, hash_key: &str, field: &str, value: &T) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        let serialized_value = serde_json::to_string(value)?;
        let redis_result: i64 = connection.hset(hash_key, field, serialized_value).await?;
        Ok(redis_result == 1)
    }

    pub async fn get_or_set_hset<T, F, Fut>(&mut self, hash_key: &str, field: &str, fetch_fn: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        let cached_str: Option<String> = connection.hget(hash_key, field).await?;

        if let Some(value_str) = cached_str {
            return Ok(serde_json::from_str(&value_str)?);
        }

        let fresh_value = fetch_fn().await?;
        let serialized_fresh_value = serde_json::to_string(&fresh_value)?;

        self.client
            .get_multiplexed_async_connection()
            .await?
            .hset::<&str, &str, String, ()>(hash_key, field, serialized_fresh_value)
            .await?;

        Ok(fresh_value)
    }

    pub async fn can_process_now(&mut self, hash_key: &str, key: &str, timeout_seconds: u64) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let last_processed_time: u64 = self.get_or_set_hset(hash_key, key, || async { Ok(now) }).await?;
        if last_processed_time == now {
            return Ok(true);
        }

        if now > last_processed_time + timeout_seconds {
            return self.set_hset(hash_key, key, &now).await;
        }

        Ok(false)
    }
}
