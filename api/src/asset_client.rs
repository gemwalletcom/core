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
        Ok(self.database.get_asset(asset_id.to_string())?.as_primitive())
    }

    pub fn get_assets_search(&mut self, query: &str) -> Result<Vec<primitives::Asset>, Box<dyn Error>> {
        let assets = self.database.get_assets_search(query)?.into_iter().map(|asset| asset.as_primitive()).collect();
        Ok(assets)
    }
}