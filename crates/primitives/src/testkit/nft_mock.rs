use crate::{
    Chain, NFTType,
    asset_constants::ETHEREUM_USDT_TOKEN_ID,
    nft::{NFTAsset, NFTAssetId, NFTImages, NFTResource},
};

const TON_NFT_COLLECTION_ADDRESS: &str = "EQCA14o1-VWhS2efqoh_9M1b_A9DtKTuoqfmkn83AbJzwnPi";
const TON_NFT_ITEM_ADDRESS: &str = "EQCvxJy4eG8hyHBFsZ7eePxrRsUQSEUTP46abUQGAcGY6mOw";

impl NFTAssetId {
    pub fn mock() -> Self {
        NFTAssetId::new(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID, "1")
    }

    pub fn mock_ton() -> Self {
        NFTAssetId::new(Chain::Ton, TON_NFT_COLLECTION_ADDRESS, TON_NFT_ITEM_ADDRESS)
    }
}

impl NFTAsset {
    pub fn mock() -> Self {
        Self::mock_with_type(NFTType::ERC721)
    }

    pub fn mock_with_type(token_type: NFTType) -> Self {
        let id = NFTAssetId::new(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID, "1");
        let collection_id = id.get_collection_id();
        NFTAsset {
            id,
            collection_id,
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

    pub fn mock_ton() -> Self {
        let id = NFTAssetId::mock_ton();
        NFTAsset {
            id: id.clone(),
            collection_id: id.get_collection_id(),
            contract_address: Some(id.token_id.clone()),
            token_id: id.token_id.clone(),
            token_type: NFTType::JETTON,
            name: "TON NFT".to_string(),
            description: None,
            chain: Chain::Ton,
            resource: NFTResource::new(String::new(), String::new()),
            images: NFTImages {
                preview: NFTResource::new(String::new(), String::new()),
            },
            attributes: vec![],
        }
    }
}
