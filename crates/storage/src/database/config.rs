use diesel::prelude::*;

use crate::DatabaseClient;
use crate::models::ConfigRow;

pub trait ConfigStore {
    fn get_config_key(&mut self, key: &str) -> Result<ConfigRow, diesel::result::Error>;
    fn add_config(&mut self, configs: Vec<ConfigRow>) -> Result<usize, diesel::result::Error>;
    fn set_config(&mut self, config_key: &str, config_value: &str) -> Result<usize, diesel::result::Error>;
}

impl ConfigStore for DatabaseClient {
    fn get_config_key(&mut self, config_key: &str) -> Result<ConfigRow, diesel::result::Error> {
        use crate::schema::config::dsl::*;
        config.filter(key.eq(config_key)).select(ConfigRow::as_select()).first(&mut self.connection)
    }

    fn add_config(&mut self, configs: Vec<ConfigRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::config::dsl::*;
        diesel::insert_into(config).values(&configs).on_conflict_do_nothing().execute(&mut self.connection)
    }

    fn set_config(&mut self, config_key: &str, config_value: &str) -> Result<usize, diesel::result::Error> {
        use crate::schema::config::dsl::*;
        diesel::update(config.filter(key.eq(config_key))).set(value.eq(config_value)).execute(&mut self.connection)
    }
}
