use primitives::ConfigKey;

use crate::DatabaseClient;
use crate::DatabaseError;
use crate::database::config::ConfigStore;
use crate::models::ConfigRow;

pub trait ConfigRepository {
    fn get_config_value(&mut self, key: ConfigKey) -> Result<String, DatabaseError>;
    fn get_config_value_i64(&mut self, key: ConfigKey) -> Result<i64, DatabaseError>;
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

    fn add_config(&mut self, configs: Vec<ConfigRow>) -> Result<usize, DatabaseError> {
        Ok(ConfigStore::add_config(self, configs)?)
    }
}
