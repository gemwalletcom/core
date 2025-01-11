use std::error::Error;

use nft::NFT;
use primitives::{Chain, NFTResult};
use std::collections::HashMap;
use storage::DatabaseClient;

pub struct NFTClient {
    database: DatabaseClient,
    nft: NFT,
}

impl NFTClient {
    pub async fn new(database_url: &str, nftscan_key: &str) -> Self {
        Self {
            database: DatabaseClient::new(database_url),
            nft: NFT::new(nftscan_key),
        }
    }

    pub async fn get_nft_assets(&mut self, device_id: &str, wallet_index: i32) -> Result<Vec<NFTResult>, Box<dyn Error>> {
        let subscriptions = self.get_subscriptions(device_id, wallet_index)?;
        let addresses: HashMap<Chain, String> = subscriptions.into_iter().map(|x| (x.chain, x.address)).collect();
        self.nft.get_assets(addresses).await
    }

    pub fn get_subscriptions(&mut self, device_id: &str, wallet_index: i32) -> Result<Vec<primitives::Subscription>, Box<dyn Error>> {
        let subscriptions = self
            .database
            .get_subscriptions_by_device_id_wallet_index(device_id, wallet_index)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect();
        Ok(subscriptions)
    }

    pub async fn get_nft_assets_by_chain(&mut self, chain: Chain, address: &str) -> Result<Vec<NFTResult>, Box<dyn Error>> {
        let addresses: HashMap<Chain, String> = [(chain, address.to_string())].iter().cloned().collect();
        self.nft.get_assets(addresses).await
    }
}
