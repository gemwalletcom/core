use std::error::Error;

use primitives::{Chain, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId};

use super::mapper::{map_asset, map_assets, map_collection};
use crate::provider::NFTProvider;
use crate::providers::opensea::client::OpenSeaClient;

#[async_trait::async_trait]
impl NFTProvider for OpenSeaClient {
    fn name(&self) -> &'static str {
        "OpenSea"
    }

    fn get_chains(&self) -> Vec<Chain> {
        vec![Chain::Ethereum, Chain::Polygon]
    }

    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        Ok(map_assets(self.get_nfts_by_account(chain, &address).await?, chain))
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let collection = self.get_collection_by_contract(collection_id.chain, &collection_id.contract_address).await?;
        Ok(map_collection(collection, collection_id))
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        map_asset(
            self.get_asset_id(asset_id.chain, &asset_id.contract_address, &asset_id.token_id).await?,
            asset_id,
        )
        .ok_or("Asset not found".into())
    }
}

#[cfg(all(test, feature = "nft_integration_tests"))]
mod nft_integration_tests {
    use crate::NFTProvider;
    use crate::testkit::*;
    use primitives::{Chain, NFTAssetId, NFTCollectionId};

    #[tokio::test]
    async fn test_opensea_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_opensea_test_client();

        let assets = client.get_assets(Chain::Ethereum, TEST_ETHEREUM_ADDRESS.to_string()).await?;

        println!("Found {} OpenSea assets", assets.len());
        assert!(!assets.is_empty());

        if let Some(asset_id) = assets.first() {
            assert_eq!(asset_id.chain, Chain::Ethereum);
            assert!(!asset_id.contract_address.is_empty());
            assert!(!asset_id.token_id.is_empty());
            println!("Sample OpenSea asset: {:?}", asset_id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_opensea_get_collection() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_opensea_test_client();

        let collection_id = NFTCollectionId::new(Chain::Ethereum, TEST_ETHEREUM_CONTRACT_ADDRESS);
        let collection = client.get_collection(collection_id).await?;

        println!("OpenSea collection: {:?}", collection);
        assert_eq!(collection.chain, Chain::Ethereum);
        assert!(!collection.name.is_empty());
        assert!(!collection.contract_address.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_opensea_get_asset() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_opensea_test_client();

        let asset_id = NFTAssetId::new(Chain::Ethereum, TEST_ETHEREUM_CONTRACT_ADDRESS, "1");
        let asset = client.get_asset(asset_id).await?;

        println!("OpenSea asset: {:?}", asset);
        assert_eq!(asset.chain, Chain::Ethereum);
        assert!(!asset.name.is_empty());
        assert!(!asset.contract_address.clone().unwrap().is_empty());
        assert_eq!(asset.token_id, "1");

        Ok(())
    }
}
