use std::collections::HashMap;

use gem_ton::models::{NftCollectionsResponse, NftItem, NftItemsResponse, TokenInfo, TokenMetadata};
use primitives::{Chain, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId, NFTData, NFTImages, NFTResource, NFTType, VerificationStatus};

use super::verified::is_verified;

pub fn map_asset_ids(response: &NftItemsResponse) -> Vec<NFTAssetId> {
    response
        .nft_items
        .iter()
        .filter(|item| valid_named_token_info(response.metadata.get(&item.address)).is_some())
        .filter_map(asset_id_from_item)
        .collect()
}

pub fn map_asset(response: NftItemsResponse, asset_id: NFTAssetId) -> Option<NFTAsset> {
    let item = response.nft_items.into_iter().next()?;
    let info = valid_named_token_info(response.metadata.get(&item.address))?;
    let collection_image = valid_named_token_info(response.metadata.get(&asset_id.contract_address)).and_then(|i| i.image.as_deref());
    Some(build_asset(&item.address, asset_id, info, collection_image))
}

pub fn map_collection(response: NftCollectionsResponse, collection_id: NFTCollectionId) -> Option<NFTCollection> {
    let collection = response.nft_collections.into_iter().next()?;
    let info = valid_named_token_info(response.metadata.get(&collection.address))?;
    Some(build_collection(&collection_id, info))
}

pub fn map_nft_data(response: NftItemsResponse) -> Vec<NFTData> {
    let NftItemsResponse { nft_items, metadata } = response;

    let collections: HashMap<NFTCollectionId, NFTCollection> = nft_items
        .iter()
        .filter_map(|item| {
            let address = item.collection_address.as_deref()?;
            let collection_id = NFTCollectionId::new(Chain::Ton, address);
            if !is_verified(&collection_id.contract_address) {
                return None;
            }
            let info = valid_named_token_info(metadata.get(&collection_id.contract_address))?;
            Some((collection_id.clone(), build_collection(&collection_id, info)))
        })
        .collect();

    nft_items
        .into_iter()
        .filter_map(|item| {
            let asset_id = asset_id_from_item(&item)?;
            let collection = collections.get(&asset_id.get_collection_id())?;
            let info = valid_named_token_info(metadata.get(&item.address))?;
            let key = asset_id.get_collection_id();
            let asset = build_asset(&item.address, asset_id, info, Some(&collection.images.preview.url));
            Some((key, asset))
        })
        .fold(HashMap::<NFTCollectionId, Vec<NFTAsset>>::new(), |mut acc, (key, asset)| {
            acc.entry(key).or_default().push(asset);
            acc
        })
        .into_iter()
        .filter_map(|(collection_id, assets)| {
            let collection = collections.get(&collection_id)?.clone();
            Some(NFTData { collection, assets })
        })
        .collect()
}

fn build_asset(item_address: &str, asset_id: NFTAssetId, info: &TokenInfo, collection_image: Option<&str>) -> NFTAsset {
    let image = info.image.as_deref().or(collection_image).unwrap_or_default();
    NFTAsset {
        id: asset_id.to_string(),
        collection_id: asset_id.get_collection_id().id(),
        contract_address: Some(item_address.to_string()),
        token_id: asset_id.token_id.clone(),
        token_type: NFTType::JETTON,
        name: token_info_name(info).unwrap_or_default().to_string(),
        description: info.description.clone(),
        chain: asset_id.chain,
        resource: NFTResource::from_url(image),
        images: NFTImages {
            preview: NFTResource::from_url(image),
        },
        attributes: vec![],
    }
}

fn build_collection(collection_id: &NFTCollectionId, info: &TokenInfo) -> NFTCollection {
    let image = info.image.clone().unwrap_or_default();
    let verified = is_verified(&collection_id.contract_address);
    NFTCollection {
        id: collection_id.id(),
        name: token_info_name(info).unwrap_or_default().to_string(),
        symbol: None,
        description: info.description.clone(),
        chain: collection_id.chain,
        contract_address: collection_id.contract_address.clone(),
        images: NFTImages {
            preview: NFTResource::from_url(&image),
        },
        status: VerificationStatus::from_verified(verified),
        links: vec![],
        is_verified: verified,
    }
}

fn valid_named_token_info(metadata: Option<&TokenMetadata>) -> Option<&TokenInfo> {
    metadata?.token_info.iter().find(|info| info.valid && token_info_name(info).is_some())
}

fn token_info_name(info: &TokenInfo) -> Option<&str> {
    info.name
        .as_deref()
        .or_else(|| info.extra.as_ref().and_then(|e| e.domain.as_deref()))
        .filter(|s| !s.is_empty())
}

fn asset_id_from_item(item: &NftItem) -> Option<NFTAssetId> {
    let collection = item.collection_address.as_deref()?;
    Some(NFTAssetId::new(Chain::Ton, collection, &item.address))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERIFIED_COLLECTION_ADDRESS: &str = "0:80D78A35F955A14B679FAA887FF4CD5BFC0F43B4A4EEA2A7E6927F3701B273C2";
    const ITEM_ADDRESS: &str = "0:AFC49CB8786F21C87045B19EDE78FC6B46C5104845133F8E9A6D440601C198EA";
    const NUMBERS_COLLECTION_ADDRESS: &str = "0:0E41DC1DC3C9067ED24248580E12B3359818D83DEE0304FABCF80845EAFAFDB2";
    const INVALID_COLLECTION_ADDRESS: &str = "0:4186117A3BE8DF8B54C4175AECEA911ACA7123845ADC8D4082D837D4C33278A5";

    #[test]
    fn test_map_asset_ids() {
        let response: NftItemsResponse = serde_json::from_str(include_str!("../../../testdata/ton/items.json")).unwrap();
        let asset_ids = map_asset_ids(&response);

        assert_eq!(asset_ids.len(), 1);
        let first = &asset_ids[0];
        assert_eq!(first.chain, Chain::Ton);
        assert_eq!(first.contract_address, VERIFIED_COLLECTION_ADDRESS);
        assert_eq!(first.token_id, ITEM_ADDRESS);
    }

    #[test]
    fn test_map_asset() {
        let response: NftItemsResponse = serde_json::from_str(include_str!("../../../testdata/ton/items.json")).unwrap();
        let asset_id = NFTAssetId::new(Chain::Ton, VERIFIED_COLLECTION_ADDRESS, ITEM_ADDRESS);
        let asset = map_asset(response, asset_id).expect("Failed to map asset");

        assert_eq!(asset.chain, Chain::Ton);
        assert_eq!(asset.token_id, ITEM_ADDRESS);
        assert_eq!(asset.contract_address.as_deref(), Some(ITEM_ADDRESS));
        assert_eq!(asset.collection_id, format!("ton_{}", VERIFIED_COLLECTION_ADDRESS));
        assert_eq!(asset.name, "Resolved Item Name");
        assert_eq!(asset.token_type, NFTType::JETTON);
        assert_eq!(asset.images.preview.url, "https://example.com/resolved-item.png");
    }

    #[test]
    fn test_map_collection() {
        let response: NftCollectionsResponse = serde_json::from_str(include_str!("../../../testdata/ton/collections.json")).unwrap();
        let collection_id = NFTCollectionId::new(Chain::Ton, NUMBERS_COLLECTION_ADDRESS);
        let collection = map_collection(response, collection_id).expect("Failed to map collection");

        assert_eq!(collection.chain, Chain::Ton);
        assert_eq!(collection.contract_address, NUMBERS_COLLECTION_ADDRESS);
        assert_eq!(collection.name, "Anonymous Telegram Numbers");
        assert!(collection.is_verified);
    }

    #[test]
    fn test_map_collection_invalid_metadata_is_rejected() {
        let response: NftCollectionsResponse = serde_json::from_str(include_str!("../../../testdata/ton/collections_invalid.json")).unwrap();
        let collection_id = NFTCollectionId::new(Chain::Ton, INVALID_COLLECTION_ADDRESS);
        assert!(map_collection(response, collection_id).is_none());
    }
}
