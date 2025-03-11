use std::error::Error;

use cacher::CacherClient;
use primitives::Markets;
use storage::DatabaseClient;

pub struct MarketsClient {
    database: DatabaseClient,
    cacher: CacherClient,
}

const MARKETS_KEY: &str = "markets";

impl MarketsClient {
    pub fn new(database_url: &str, redis_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        let cacher = CacherClient::new(redis_url);
        Self { database, cacher }
    }

    pub async fn get_markets(&mut self) -> Result<Markets, Box<dyn Error + Send + Sync>> {
        self.cacher.get_serialized_value::<Markets>(MARKETS_KEY).await
    }

    pub async fn set_markets(&mut self, markets: Markets) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.set_serialized_value(MARKETS_KEY, &markets).await
    }

    pub async fn get_asset_ids_for_prices_ids(&mut self, price_ids: Vec<String>) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let assets = self.database.get_prices_assets_for_price_ids(price_ids)?;
        Ok(assets.into_iter().map(|x| x.asset_id).collect())
    }
}
