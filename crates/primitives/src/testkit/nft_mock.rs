use crate::{
    Chain, NFTType,
    asset_constants::ETHEREUM_USDT_TOKEN_ID,
    nft::{NFTAsset, NFTImages, NFTResource},
};

impl NFTAsset {
    pub fn mock() -> Self {
        Self::mock_with_type(NFTType::ERC721)
    }

    pub fn mock_with_type(token_type: NFTType) -> Self {
        NFTAsset {
            id: "nft_1".to_string(),
            collection_id: String::new(),
            contract_address: Some(ETHEREUM_USDT_TOKEN_ID.to_string()),
            token_id: "1".to_string(),
            token_type,
            name: "Test NFT".to_string(),
            description: None,
            chain: Chain::Ethereum,
            resource: NFTResource::new(String::new(), String::new()),
            images: NFTImages {
                preview: NFTResource::new(String::new(), String::new()),
            },
            attributes: vec![],
        }
    }
}
