use std::error::Error;

use redis::{AsyncCommands, Client};

// Work in progress. In the future use it for caching any temporary data.
pub struct CacherClient {
    client: Client,
}

impl CacherClient {
    pub fn new(redis_url: &str) -> Self {
        let client = Client::open(redis_url).unwrap();
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

    pub async fn set_value_with_expiration(&mut self, key: &str, value: String, seconds: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        connection.set::<&str, String, ()>(key, value).await?;
        connection.expire::<&str, String>(key, seconds).await?;
        Ok(())
    }

    pub async fn get_string(&mut self, key: &str) -> Result<String, Box<dyn Error>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        Ok(connection.get(key).await?)
    }

    pub async fn set_value<T: serde::Serialize>(&mut self, key: &str, value: &T) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        Ok(connection.set::<&str, String, ()>(key, serde_json::to_string(value)?).await?)
    }

    pub async fn get_value<T: serde::de::DeserializeOwned>(&mut self, key: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        let value: String = connection.get(key).await?;
        Ok(serde_json::from_str(&value)?)
    }

    pub async fn get_or_set_value<T, F, Fut>(&mut self, key: &str, fetch_fn: F, ttl_seconds: Option<i64>) -> Result<T, Box<dyn Error + Send + Sync>>
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
            self.set_value_with_expiration(key, serialized, ttl).await?;
        } else {
            connection.set::<&str, String, ()>(key, serialized).await?;
        }

        Ok(fresh_value)
    }
}
