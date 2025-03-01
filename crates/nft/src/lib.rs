use std::collections::HashMap;

use async_trait::async_trait;
use nftscan::client::NFTScanClient;
use primitives::{nft::NFTCollectionId, Chain, NFTAsset, NFTAssetId, NFTCollection};

pub mod magiceden;
pub mod nftscan;
pub mod opensea;
pub use magiceden::MagicEdenClient;
pub use opensea::OpenSeaClient;

#[async_trait]
pub trait NFTProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_collection(&self, collection: NFTCollectionId) -> Result<NFTCollection, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn std::error::Error + Send + Sync>>;
}

#[allow(unused)]
pub struct NFT {
    nftscan_client: NFTScanClient,
    opensea_client: OpenSeaClient,
    magiceden_client: MagicEdenClient,
}

impl NFT {
    pub fn new(nftscan_key: &str, opensea_key: &str, magiceden_key: &str) -> Self {
        Self {
            nftscan_client: NFTScanClient::new(nftscan_key),
            opensea_client: OpenSeaClient::new(opensea_key),
            magiceden_client: MagicEdenClient::new(magiceden_key),
        }
    }

    pub async fn get_assets(&self, addresses: HashMap<Chain, String>) -> Result<Vec<NFTAssetId>, Box<dyn std::error::Error + Send + Sync>> {
        let futures: Vec<_> = addresses
            .into_iter()
            .map(|(chain, address)| {
                let address = address.clone();
                async move { self.get_asset_ids(chain, address.as_str()).await }
            })
            .collect();

        Ok(futures::future::try_join_all(futures).await?.into_iter().flatten().collect::<Vec<NFTAssetId>>())
    }

    pub async fn get_asset_ids(&self, chain: Chain, address: &str) -> Result<Vec<NFTAssetId>, Box<dyn std::error::Error + Send + Sync>> {
        match chain {
            Chain::Ethereum => self.nftscan_client.get_assets(chain, address.to_string()).await,
            Chain::Solana => self.magiceden_client.get_assets(chain, address.to_string()).await,
            _ => Ok(vec![]),
        }
    }

    pub async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn std::error::Error + Send + Sync>> {
        match collection_id.chain {
            Chain::Ethereum => self.nftscan_client.get_collection(collection_id).await,
            Chain::Solana => self.magiceden_client.get_collection(collection_id).await,
            _ => unimplemented!(),
        }
    }

    pub async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn std::error::Error + Send + Sync>> {
        match asset_id.chain {
            Chain::Ethereum => self.nftscan_client.get_asset(asset_id).await,
            Chain::Solana => self.magiceden_client.get_asset(asset_id).await,
            _ => unimplemented!(),
        }
    }
}
