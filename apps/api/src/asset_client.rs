extern crate rocket;
use std::error::Error;

use primitives::AssetFull;
use storage::DatabaseClient;

pub struct AssetsClient {
    database: DatabaseClient,
}

impl AssetsClient {
    pub async fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub fn get_assets_list(&mut self) -> Result<Vec<AssetFull>, Box<dyn Error>> {
        let assets = self.database.get_assets_list()?;
        Ok(assets.into_iter().map(|x| x.as_primitive_full()).collect())
    }

    pub fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<AssetFull>, Box<dyn Error>> {
        let assets = self.database.get_assets(asset_ids)?;
        Ok(assets.into_iter().map(|x| x.as_primitive_full()).collect())
    }

    pub fn get_asset_full(&mut self, asset_id: &str) -> Result<AssetFull, Box<dyn Error>> {
        let asset = self.database.get_asset(asset_id)?;
        let asset_price = self.database.get_price(asset_id).ok();
        let market = asset_price.clone().map(|x| x.as_market_primitive());
        let price = asset_price.clone().clone().map(|x| x.as_price_primitive());
        let details = self
            .database
            .get_asset_details(asset_id)
            .ok()
            .map(|x| x.as_primitive());

        let score = asset.as_score_primitive();
        let asset = asset.as_primitive();
        Ok(AssetFull {
            asset,
            details,
            price,
            market,
            score,
        })
    }

    pub fn get_assets_search(
        &mut self,
        query: &str,
        chains: Vec<String>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<primitives::AssetFull>, Box<dyn Error>> {
        let min_score = if query.len() > 10 { -100 } else { 10 };
        let assets = self
            .database
            .get_assets_search(query, chains, min_score, limit, offset)?
            .into_iter()
            .map(|asset| AssetFull {
                asset: asset.as_primitive(),
                details: None,
                price: None,
                market: None,
                score: asset.as_score_primitive(),
            })
            .collect();
        Ok(assets)
    }

    pub fn get_assets_ids_by_device_id(
        &mut self,
        device_id: &str,
        wallet_index: i32,
        from_timestamp: Option<u32>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let subscriptions = self
            .database
            .get_subscriptions_by_device_id_wallet_index(device_id, wallet_index)?;

        let addresses = subscriptions
            .clone()
            .into_iter()
            .map(|x| x.address)
            .collect();
        let chains = subscriptions
            .clone()
            .into_iter()
            .map(|x| x.chain)
            .collect::<Vec<_>>();

        let assets_ids =
            self.database
                .get_assets_ids_by_device_id(addresses, chains, from_timestamp)?;
        Ok(assets_ids)
    }
}
