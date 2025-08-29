use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait SecurePreferences: Send + Sync {
    fn get(&self, key: String) -> Result<Option<String>, Box<dyn Error + Send + Sync>>;
    fn set(&self, key: String, value: String) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn remove(&self, key: String) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub trait Preferences: Send + Sync {
    fn get(&self, key: String) -> Result<Option<String>, Box<dyn Error + Send + Sync>>;
    fn set(&self, key: String, value: String) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn remove(&self, key: String) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub trait PreferencesExt {
    fn get_i64(&self, key: &str) -> Result<Option<i64>, Box<dyn Error + Send + Sync>>;
    fn set_i64(&self, key: &str, value: i64) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn get_bool(&self, key: &str) -> Result<Option<bool>, Box<dyn Error + Send + Sync>>;
    fn set_bool(&self, key: &str, value: bool) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn get_i64_with_ttl(&self, key: &str, ttl_seconds: u64) -> Result<Option<i64>, Box<dyn Error + Send + Sync>>;
    fn set_i64_with_ttl(&self, key: &str, value: i64, ttl_seconds: u64) -> Result<(), Box<dyn Error + Send + Sync>>;
}

impl<T: Preferences + ?Sized> PreferencesExt for T {
    fn get_i64(&self, key: &str) -> Result<Option<i64>, Box<dyn Error + Send + Sync>> {
        if let Some(value) = self.get(key.to_string())? {
            Ok(Some(value.parse()?))
        } else {
            Ok(None)
        }
    }

    fn set_i64(&self, key: &str, value: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(key.to_string(), value.to_string())
    }

    fn get_bool(&self, key: &str) -> Result<Option<bool>, Box<dyn Error + Send + Sync>> {
        if let Some(value) = self.get(key.to_string())? {
            Ok(Some(value.parse()?))
        } else {
            Ok(None)
        }
    }

    fn set_bool(&self, key: &str, value: bool) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(key.to_string(), value.to_string())
    }

    fn get_i64_with_ttl(&self, key: &str, ttl_seconds: u64) -> Result<Option<i64>, Box<dyn Error + Send + Sync>> {
        let timestamp_key = format!("{}_timestamp", key);

        if let (Some(value), Some(timestamp)) = (self.get(key.to_string())?, self.get(timestamp_key)?) {
            let cached_time: u64 = timestamp.parse()?;
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

            if current_time - cached_time < ttl_seconds {
                return Ok(Some(value.parse()?));
            }
        }

        Ok(None)
    }

    fn set_i64_with_ttl(&self, key: &str, value: i64, _ttl_seconds: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let timestamp_key = format!("{}_timestamp", key);
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        self.set(key.to_string(), value.to_string())?;
        self.set(timestamp_key, current_time.to_string())?;
        Ok(())
    }
}
