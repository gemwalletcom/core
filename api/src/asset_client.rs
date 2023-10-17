extern crate rocket;
use std::error::Error;

use primitives::AssetFull;
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

    pub fn get_asset_full(&mut self, asset_id: &str) -> Result<AssetFull, Box<dyn Error>> {
        let asset = self.database.get_asset(asset_id.to_string())?.as_primitive();
        let asset_price = self.database.get_price(asset_id);
        let market = asset_price.ok().map(|x| x.as_market_primitive());
        let price = asset_price.ok().map(|x| x.as_price_primitive());
        let details = self.database.get_asset_details(asset_id.to_string()).ok().map(|x| x.as_primitive());
        Ok(AssetFull{asset, details, price, market})
    }

    pub fn get_assets_search(&mut self, query: &str) -> Result<Vec<primitives::Asset>, Box<dyn Error>> {
        let assets = self.database.get_assets_search(query)?.into_iter().map(|asset| asset.as_primitive()).collect();
        Ok(assets)
    }
}