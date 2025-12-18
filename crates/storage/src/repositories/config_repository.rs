use std::time::Duration;

use primitives::{ConfigKey, parse_duration};

use crate::DatabaseClient;
use crate::DatabaseError;
use crate::database::config::ConfigStore;
use crate::models::ConfigRow;

pub trait ConfigRepository {
    fn get_config_value(&mut self, key: ConfigKey) -> Result<String, DatabaseError>;
    fn get_config_value_i64(&mut self, key: ConfigKey) -> Result<i64, DatabaseError>;
    fn get_config_value_f64(&mut self, key: ConfigKey) -> Result<f64, DatabaseError>;
    fn get_config_value_bool(&mut self, key: ConfigKey) -> Result<bool, DatabaseError>;
    fn get_config_value_duration(&mut self, key: ConfigKey) -> Result<Duration, DatabaseError>;
    fn add_config(&mut self, configs: Vec<ConfigRow>) -> Result<usize, DatabaseError>;
}

impl ConfigRepository for DatabaseClient {
    fn get_config_value(&mut self, key: ConfigKey) -> Result<String, DatabaseError> {
        let result = ConfigStore::get_config(self, key.as_ref())?;
        Ok(result.value)
    }

    fn get_config_value_i64(&mut self, key: ConfigKey) -> Result<i64, DatabaseError> {
        let value = self.get_config_value(key)?;
        value
            .parse()
            .map_err(|e| DatabaseError::Internal(format!("Failed to parse config value: {}", e)))
    }

    fn get_config_value_f64(&mut self, key: ConfigKey) -> Result<f64, DatabaseError> {
        let value = self.get_config_value(key)?;
        value
            .parse()
            .map_err(|e| DatabaseError::Internal(format!("Failed to parse config value: {}", e)))
    }

    fn get_config_value_bool(&mut self, key: ConfigKey) -> Result<bool, DatabaseError> {
        let value = self.get_config_value(key)?;
        value
            .parse()
            .map_err(|e| DatabaseError::Internal(format!("Failed to parse config value: {}", e)))
    }

    fn get_config_value_duration(&mut self, key: ConfigKey) -> Result<Duration, DatabaseError> {
        let value = self.get_config_value(key)?;
        parse_duration(&value).ok_or_else(|| DatabaseError::Internal(format!("Failed to parse duration: {}", value)))
    }

    fn add_config(&mut self, configs: Vec<ConfigRow>) -> Result<usize, DatabaseError> {
        Ok(ConfigStore::add_config(self, configs)?)
    }
}
