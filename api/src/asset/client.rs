extern crate rocket;
use std::error::Error;

use storage::DatabaseClient;

pub struct AssetsClient {
    database: DatabaseClient,
}

impl AssetsClient {
    pub async fn new(
        database_url: &str
    ) -> Self {
        let database = DatabaseClient::new(database_url);
        Self {
            database,
        }
    }

    pub fn get_asset(&mut self, asset_id: &str) -> Result<primitives::Asset, Box<dyn Error>> {
        return Ok(self.database.get_asset(asset_id.to_string())?.as_primitive())
    }
}