use std::error::Error;

use primitives::{Chain, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId};

use super::mapper::{map_asset, map_assets, map_collection};
use crate::provider::NFTProvider;
use crate::providers::magiceden::client::MagicEdenClient;

#[async_trait::async_trait]
impl NFTProvider for MagicEdenClient {
    fn name(&self) -> &'static str {
        "MagicEden"
    }

    fn get_chains(&self) -> Vec<Chain> {
        vec![Chain::Solana]
    }

    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        let response = self.get_nfts_by_account(&address).await?;
        Ok(map_assets(response, chain))
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let collection = self.get_collection_id(&collection_id.contract_address).await?;
        Ok(map_collection(collection, collection_id))
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let nft = self.get_asset_id(&asset_id.token_id).await?;
        Ok(map_asset(nft.clone(), asset_id, nft.owner.clone()).ok_or("Asset not found")?)
    }
}

#[cfg(all(test, feature = "nft_integration_tests"))]
mod nft_integration_tests {
    use crate::NFTProvider;
    use crate::testkit::*;
    use primitives::{Chain, NFTAssetId, NFTCollectionId};

    #[tokio::test]
    async fn test_magiceden_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_magiceden_test_client();

        let assets = client.get_assets(Chain::Solana, TEST_SOLANA_ADDRESS.to_string()).await?;

        println!("Found {} MagicEden assets", assets.len());
        assert!(!assets.is_empty());

        if let Some(asset_id) = assets.first() {
            assert_eq!(asset_id.chain, Chain::Solana);
            assert!(!asset_id.token_id.is_empty());
            println!("Sample MagicEden asset: {:?}", asset_id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_magiceden_get_collection() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_magiceden_test_client();

        let collection_id = NFTCollectionId::new(Chain::Solana, TEST_SOLANA_COLLECTION);
        let collection = client.get_collection(collection_id).await?;

        println!("MagicEden collection: {:?}", collection);
        assert_eq!(collection.chain, Chain::Solana);
        assert!(!collection.name.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_magiceden_get_asset() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_magiceden_test_client();

        let asset_id = NFTAssetId::new(Chain::Solana, TEST_SOLANA_COLLECTION_POOKS, TEST_SOLANA_TOKEN_ID);

        let asset = client.get_asset(asset_id).await?;
        println!("MagicEden asset: {:?}", asset);

        assert_eq!(asset.id, format!("solana_{}_{}", TEST_SOLANA_COLLECTION_POOKS, TEST_SOLANA_TOKEN_ID));
        assert_eq!(asset.chain, Chain::Solana);
        assert!(!asset.name.is_empty());
        assert!(!asset.attributes.is_empty());
        assert_eq!(asset.token_id, TEST_SOLANA_TOKEN_ID);

        Ok(())
    }
}
