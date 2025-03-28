use std::error::Error;

use primitives::NFTAsset;
use primitives::NFTAssetId;
use primitives::NFTCollectionId;
use primitives::{Chain, NFTCollection};

use super::client::NFTScanClient;
use super::model::get_chain;
use crate::NFTProvider;

#[async_trait::async_trait]
impl NFTProvider for NFTScanClient {
    fn name(&self) -> &'static str {
        "OpenSea"
    }

    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        let chain_str = get_chain(chain).ok_or("Chain not supported")?;
        match chain {
            Chain::Ethereum => Ok(self
                .get_evm_nfts(vec![chain_str], address.as_str())
                .await?
                .data
                .into_iter()
                .flat_map(|nft| nft.as_primitive())
                .collect()),
            Chain::Solana => Ok(self
                .get_solana_nfts(address.as_str())
                .await?
                .data
                .into_iter()
                .flat_map(|nft| nft.as_primitive_asset_ids())
                .collect()),
            _ => Ok(vec![]),
        }
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let chain = get_chain(collection_id.chain).ok_or("Chain not supported")?;
        match collection_id.chain {
            Chain::Ethereum => Ok(self
                .get_collection_id(&chain, &collection_id.contract_address)
                .await?
                .data
                .as_primitive(collection_id.chain)
                .ok_or("asset not found")?),
            Chain::Solana => panic!("Not implemented"),
            _ => panic!("Not implemented"),
        }
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        match asset_id.chain {
            Chain::Ethereum => Ok(self
                .get_asset_id(&asset_id.contract_address, &asset_id.token_id)
                .await?
                .data
                .as_primitive(asset_id.chain, &asset_id.contract_address)
                .ok_or("asset not found")?),
            Chain::Solana => Ok(self
                .get_solana_asset_id(&asset_id.token_id)
                .await?
                .data
                .as_primitive(asset_id.chain, &asset_id.contract_address)
                .ok_or("asset not found")?),
            _ => panic!("Not implemented"),
        }
    }
}
