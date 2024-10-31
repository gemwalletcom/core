extern crate rocket;

use std::error::Error;

use primitives::FiatAssets;
use storage::DatabaseClient;

pub struct SwapClient {
    database: DatabaseClient,
}

impl SwapClient {
    pub async fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub async fn get_swap_assets(&mut self) -> Result<FiatAssets, Box<dyn Error>> {
        let assets = self.database.get_swap_assets()?;
        let version = self.database.get_swap_assets_version()?;

        Ok(FiatAssets {
            version: version as u32,
            asset_ids: assets,
        })
    }
}
