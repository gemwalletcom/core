use simplehash::client::{SIMPLEHASH_EVM_CHAINS, SIMPLEHASH_SOLANA_CHAIN};
use std::collections::HashMap;

use nftscan::NFTScanClient;
use primitives::Chain;

pub mod nftscan;
pub mod opensea;
pub use opensea::OpenSeaClient;
pub mod simplehash;
pub use simplehash::SimpleHashClient;

pub struct NFT {
    nftscan_client: NFTScanClient,
    simplehash_client: SimpleHashClient,
}

impl NFT {
    pub fn new(nftscan_key: &str, simplehash_key: &str) -> Self {
        Self {
            nftscan_client: NFTScanClient::new(nftscan_key),
            simplehash_client: SimpleHashClient::new(simplehash_key.to_string()),
        }
    }

    pub async fn get_assets(&self, addresses: HashMap<Chain, String>) -> Result<Vec<primitives::NFTData>, Box<dyn std::error::Error + Send + Sync>> {
        let futures: Vec<_> = addresses
            .into_iter()
            .map(|(chain, address)| {
                let address = address.clone();
                async move { self.get_nfts(chain, address.as_str()).await }
            })
            .collect();

        let assets = futures::future::try_join_all(futures)
            .await?
            .into_iter()
            .flatten()
            .filter(|x| x.collection.is_verified)
            .filter(|x| !x.assets.is_empty())
            .collect();

        Ok(assets)
    }

    pub async fn get_nfts(&self, chain: Chain, address: &str) -> Result<Vec<primitives::NFTData>, reqwest::Error> {
        let pages_limit = 5;
        match chain {
            Chain::Ethereum => self
                .simplehash_client
                .get_assets_all(address, SIMPLEHASH_EVM_CHAINS.to_vec(), pages_limit)
                .await
                .map(|x| x.as_primitives()),
            Chain::Ton => self.nftscan_client.get_ton_nfts(address).await.map(|x| {
                x.data
                    .into_iter()
                    .filter_map(|result| {
                        result.as_primitive().map(|collection| primitives::NFTData {
                            collection: collection.clone(),
                            assets: result.assets.into_iter().filter_map(|x| x.as_primitive(&collection.id)).collect(),
                        })
                    })
                    .collect::<Vec<_>>()
            }),
            Chain::Solana => self
                .simplehash_client
                .get_assets_all(address, SIMPLEHASH_SOLANA_CHAIN.to_vec(), pages_limit)
                .await
                .map(|x| x.as_primitives()),
            _ => Ok(vec![]),
        }
    }
}
