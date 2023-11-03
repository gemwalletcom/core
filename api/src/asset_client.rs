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
        let asset = self.database.get_asset(asset_id.to_string())?;
        let asset_price = self.database.get_price(asset_id).ok();
        let market = asset_price.clone().map(|x| x.as_market_primitive());
        let price = asset_price.clone().clone().map(|x| x.as_price_primitive());
        let details = self.database.get_asset_details(asset_id.to_string()).ok().map(|x| x.as_primitive());
        let score = asset.as_score_primitive();
        let asset = asset.as_primitive();
        Ok(AssetFull{asset, details, price, market, score})
    }

    pub fn get_assets_search(&mut self, query: &str) -> Result<Vec<primitives::AssetFull>, Box<dyn Error>> {
        let assets = self.database.get_assets_search(query)?.into_iter().map(|asset| 
            AssetFull{
                asset: asset.as_primitive(), 
                details: None,
                price: None,
                market: None,
                score: asset.as_score_primitive(),
            }
        ).collect();
        Ok(assets)
    }
}