use std::error::Error;

use redis::AsyncCommands;

// Work in progress. In the future use it for caching any temporary data.
pub struct CacherClient {
    client: redis::Client,
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

    pub async fn set_value_with_expiration(&mut self, key: &str, value: String, seconds: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        connection.set::<&str, String, ()>(key, value).await?;
        connection.expire::<&str, String>(key, seconds).await?;
        Ok(())
    }

    pub async fn get_value(&mut self, key: &str) -> Result<String, Box<dyn Error>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        let value: String = connection.get(key).await?;
        Ok(value)
    }
}
