use std::time::Duration;

use primitives::{ConfigKey, parse_duration};

use crate::DatabaseClient;
use crate::DatabaseError;
use crate::database::config::ConfigStore;
use crate::models::ConfigRow;

pub trait ConfigRepository {
    fn get_config(&mut self, key: ConfigKey) -> Result<String, DatabaseError>;
    fn get_config_i64(&mut self, key: ConfigKey) -> Result<i64, DatabaseError>;
    fn get_config_f64(&mut self, key: ConfigKey) -> Result<f64, DatabaseError>;
    fn get_config_bool(&mut self, key: ConfigKey) -> Result<bool, DatabaseError>;
    fn get_config_duration(&mut self, key: ConfigKey) -> Result<Duration, DatabaseError>;
    fn get_config_vec_string(&mut self, key: ConfigKey) -> Result<Vec<String>, DatabaseError>;
    fn add_config(&mut self, configs: Vec<ConfigRow>) -> Result<usize, DatabaseError>;
}

impl ConfigRepository for DatabaseClient {
    fn get_config(&mut self, key: ConfigKey) -> Result<String, DatabaseError> {
        let result = ConfigStore::get_config_key(self, key.as_ref())?;
        Ok(result.value)
    }

    fn get_config_i64(&mut self, key: ConfigKey) -> Result<i64, DatabaseError> {
        self.get_config(key)?
            .parse()
            .map_err(|e| DatabaseError::Error(format!("Failed to parse config value: {}", e)))
    }

    fn get_config_f64(&mut self, key: ConfigKey) -> Result<f64, DatabaseError> {
        self.get_config(key)?
            .parse()
            .map_err(|e| DatabaseError::Error(format!("Failed to parse config value: {}", e)))
    }

    fn get_config_bool(&mut self, key: ConfigKey) -> Result<bool, DatabaseError> {
        self.get_config(key)?
            .parse()
            .map_err(|e| DatabaseError::Error(format!("Failed to parse config value: {}", e)))
    }

    fn get_config_duration(&mut self, key: ConfigKey) -> Result<Duration, DatabaseError> {
        let value = self.get_config(key)?;
        parse_duration(&value).ok_or_else(|| DatabaseError::Error(format!("Failed to parse duration: {}", value)))
    }

    fn get_config_vec_string(&mut self, key: ConfigKey) -> Result<Vec<String>, DatabaseError> {
        let value = self.get_config(key)?;
        serde_json::from_str(&value).map_err(|e| DatabaseError::Error(format!("Failed to parse config vec: {}", e)))
    }

    fn add_config(&mut self, configs: Vec<ConfigRow>) -> Result<usize, DatabaseError> {
        Ok(ConfigStore::add_config(self, configs)?)
    }
}
