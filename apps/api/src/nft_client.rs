use std::error::Error;

use primitives::{NFTCollectible, NFTCollection};
use storage::DatabaseClient;

pub struct NFTClient {
    database: DatabaseClient,
}

impl NFTClient {
    pub async fn new(database_url: &str) -> Self {
        Self {
            database: DatabaseClient::new(database_url),
        }
    }

    pub async fn get_nft_collections(
        &mut self,
        device_id: &str,
        wallet_index: i32,
    ) -> Result<Vec<NFTCollection>, Box<dyn Error>> {
        let _subscriptions = self.get_subscriptions(device_id, wallet_index)?;
        //TODO: subscriptions contains all of the addresses of the user. Fetch across all chains their nft
        Ok(vec![])
    }

    pub async fn get_nft_collectibles(
        &mut self,
        device_id: &str,
        _collection_id: &str,
        wallet_index: i32,
    ) -> Result<Vec<NFTCollectible>, Box<dyn Error>> {
        let _subscriptions = self.get_subscriptions(device_id, wallet_index)?;
        //TODO: Get all the collectibles of the user.
        Ok(vec![])
    }

    pub fn get_subscriptions(
        &mut self,
        device_id: &str,
        wallet_index: i32,
    ) -> Result<Vec<primitives::Subscription>, Box<dyn Error>> {
        let subscriptions = self
            .database
            .get_subscriptions_by_device_id_wallet_index(device_id, wallet_index)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect();
        Ok(subscriptions)
    }
}
