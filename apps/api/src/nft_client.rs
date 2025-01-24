use std::error::Error;

use nft::NFT;
use primitives::{Chain, NFTCollection, NFTData};
use std::collections::HashMap;
use storage::DatabaseClient;

pub struct NFTClient {
    database: DatabaseClient,
    nft: NFT,
}

impl NFTClient {
    pub async fn new(database_url: &str, nftscan_key: &str, simplehash_key: &str) -> Self {
        Self {
            database: DatabaseClient::new(database_url),
            nft: NFT::new(nftscan_key, simplehash_key),
        }
    }

    pub async fn update_collection(&mut self, _collection_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    pub async fn update_asset(&mut self, _asset_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    pub async fn get_nft_assets(&mut self, device_id: &str, wallet_index: i32) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.get_subscriptions(device_id, wallet_index)?;
        let addresses: HashMap<Chain, String> = subscriptions.into_iter().map(|x| (x.chain, x.address)).collect();

        let addresses: HashMap<Chain, String> = addresses
            .into_iter()
            .filter(|x| matches!(x.0, Chain::Ethereum))
            // .filter(|x| matches!(x.0, Chain::Ethereum | ChainType::Solana))
            //.filter(|x| matches!(x.0, Chain::Solana))
            .collect();

        self.get_nfts(self.nft.get_assets(addresses).await?).await
    }

    pub fn get_subscriptions(&mut self, device_id: &str, wallet_index: i32) -> Result<Vec<primitives::Subscription>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self
            .database
            .get_subscriptions_by_device_id_wallet_index(device_id, wallet_index)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect();
        Ok(subscriptions)
    }

    pub async fn get_nft_assets_by_chain(&mut self, chain: Chain, address: &str) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let addresses: HashMap<Chain, String> = [(chain, address.to_string())].iter().cloned().collect();
        self.get_nfts(self.nft.get_assets(addresses).await?).await
    }

    pub async fn get_nft_collection(&mut self, chain: Chain, id: &str) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let id = format!("{}_{}", chain.as_ref(), id);
        Ok(self.database.get_nft_collection(id.as_str())?.as_primitive())
    }

    // computed nfts from db
    async fn get_nfts(&mut self, nfts: Vec<NFTData>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        // cache collections and assets locally
        let collections = nfts
            .clone()
            .into_iter()
            .map(|x| storage::models::NftCollection::from_primitive(x.collection.clone()))
            .collect();

        self.database.add_nft_collections(collections)?;

        let assets = nfts
            .clone()
            .into_iter()
            .flat_map(|x| x.assets)
            .clone()
            .map(|x| storage::models::NftAsset::from_primitive(x.clone()))
            .collect();

        self.database.add_nft_assets(assets)?;

        let results = nfts
            .into_iter()
            .map(|x| {
                let collection = self.database.get_nft_collection(&x.collection.id).unwrap().as_primitive();
                let asset_ids = x.assets.into_iter().map(|x| x.id).collect();
                let assets = self.database.get_nft_assets(asset_ids).unwrap().into_iter().map(|x| x.as_primitive()).collect();

                NFTData {
                    collection: collection.clone(),
                    assets,
                }
            })
            .collect();
        Ok(results)
    }
}
