use std::error::Error;

use primitives::{Chain, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId};

use super::client::MagicEdenEvmClient;
use super::mapper::{map_asset, map_assets, map_collection};
use crate::provider::NFTProvider;

#[async_trait::async_trait]
impl NFTProvider for MagicEdenEvmClient {
    fn name(&self) -> &'static str {
        "MagicEdenEVM"
    }

    fn get_chains(&self) -> Vec<Chain> {
        vec![Chain::SmartChain]
    }

    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        let response = self.get_nfts_by_wallet(chain, &address).await?;
        Ok(map_assets(response.assets, chain))
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let collection = self.fetch_collection_detail(collection_id.chain, &collection_id.contract_address).await?;
        Ok(map_collection(collection, collection_id))
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let response = self.get_token(asset_id.chain, &asset_id.contract_address, &asset_id.token_id).await?;
        Ok(map_asset(response.token, asset_id).ok_or("Asset not found")?)
    }
}

#[cfg(all(test, feature = "nft_integration_tests"))]
mod nft_integration_tests {
    use crate::NFTProvider;
    use crate::testkit::*;
    use primitives::{Chain, NFTAssetId, NFTCollectionId};

    #[tokio::test]
    async fn test_magiceden_evm_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_magiceden_evm_test_client();

        let assets = client.get_assets(Chain::SmartChain, TEST_BSC_ADDRESS.to_string()).await?;

        println!("Found {} MagicEden EVM assets", assets.len());
        assert!(!assets.is_empty());

        if let Some(asset_id) = assets.first() {
            assert_eq!(asset_id.chain, Chain::SmartChain);
            assert!(!asset_id.token_id.is_empty());
            assert!(!asset_id.contract_address.is_empty());
            println!("Sample MagicEden EVM asset: {:?}", asset_id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_magiceden_evm_get_collection() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_magiceden_evm_test_client();

        let collection_id = NFTCollectionId::new(Chain::SmartChain, TEST_BSC_COLLECTION);
        let collection = client.get_collection(collection_id).await?;

        println!("MagicEden EVM collection: {:?}", collection);
        assert_eq!(collection.chain, Chain::SmartChain);
        assert!(!collection.name.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_magiceden_evm_get_asset() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_magiceden_evm_test_client();

        let asset_id = NFTAssetId::new(Chain::SmartChain, TEST_BSC_COLLECTION, "410");

        let asset = client.get_asset(asset_id).await?;
        println!("MagicEden EVM asset: {:?}", asset);

        assert_eq!(asset.id, format!("smartchain_{}_410", TEST_BSC_COLLECTION));
        assert_eq!(asset.chain, Chain::SmartChain);
        assert!(!asset.name.is_empty());
        assert!(!asset.attributes.is_empty());
        assert_eq!(asset.token_id, "410");

        Ok(())
    }
}
