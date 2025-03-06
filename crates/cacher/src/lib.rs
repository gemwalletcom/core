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

    pub async fn get_value(&mut self, key: &str) -> Result<String, Box<dyn Error>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        Ok(connection.get(key).await?)
    }
    pub async fn set_serialized_value<T: serde::Serialize>(&mut self, key: &str, value: &T) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        Ok(connection.set::<&str, String, ()>(key, serde_json::to_string(value)?).await?)
    }

    pub async fn get_serialized_value<T: serde::de::DeserializeOwned>(&mut self, key: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        let value: String = connection.get(key).await?;
        Ok(serde_json::from_str(&value)?)
    }
}
