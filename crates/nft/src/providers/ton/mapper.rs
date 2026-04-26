use std::collections::HashMap;

use gem_ton::address::Address;
use gem_ton::models::{NftCollectionsResponse, NftItem, NftItemsResponse, TokenInfo, TokenMetadata};
use primitives::{Chain, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId, NFTData, NFTImages, NFTResource, NFTType, VerificationStatus};

use super::verified::is_verified;

pub fn map_asset_ids(response: &NftItemsResponse) -> Vec<NFTAssetId> {
    response.nft_items.iter().filter_map(|item| asset_id_from_item(item, &response.metadata)).collect()
}

pub fn map_asset(response: NftItemsResponse, asset_id: NFTAssetId) -> Option<NFTAsset> {
    let collection = Address::from_base64_url(&asset_id.contract_address).ok()?;
    if !is_verified(&collection) {
        return None;
    }
    let item = response.nft_items.into_iter().next()?;
    let info = valid_named_token_info(response.metadata.get(&item.address))?;
    let collection_image = item
        .collection_address
        .as_deref()
        .and_then(|hex| valid_named_token_info(response.metadata.get(hex)))
        .and_then(|i| i.image.as_deref());
    Some(build_asset(asset_id, info, collection_image))
}

pub fn map_collection(response: NftCollectionsResponse, collection_id: NFTCollectionId) -> Option<NFTCollection> {
    let address = Address::from_base64_url(&collection_id.contract_address).ok()?;
    if !is_verified(&address) {
        return None;
    }
    let collection = response.nft_collections.into_iter().next()?;
    let info = valid_named_token_info(response.metadata.get(&collection.address))?;
    Some(build_collection(&collection_id, info))
}

pub fn map_nft_data(response: NftItemsResponse) -> Vec<NFTData> {
    let NftItemsResponse { nft_items, metadata } = response;

    let collections: HashMap<NFTCollectionId, NFTCollection> = nft_items
        .iter()
        .filter_map(|item| {
            let hex = item.collection_address.as_deref()?;
            let address = Address::from_hex_str(hex).ok()?;
            if !is_verified(&address) {
                return None;
            }
            let info = valid_named_token_info(metadata.get(hex))?;
            let collection_id = NFTCollectionId::new(Chain::Ton, &address.to_base64_url());
            Some((collection_id.clone(), build_collection(&collection_id, info)))
        })
        .collect();

    nft_items
        .into_iter()
        .filter_map(|item| {
            let asset_id = asset_id_from_item(&item, &metadata)?;
            let collection = collections.get(&asset_id.get_collection_id())?;
            let info = valid_named_token_info(metadata.get(&item.address))?;
            let key = asset_id.get_collection_id();
            let asset = build_asset(asset_id, info, Some(&collection.images.preview.url));
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

fn build_asset(asset_id: NFTAssetId, info: &TokenInfo, collection_image: Option<&str>) -> NFTAsset {
    let image = info.image.as_deref().or(collection_image).unwrap_or_default();
    NFTAsset {
        id: asset_id.to_string(),
        collection_id: asset_id.get_collection_id().id(),
        contract_address: Some(asset_id.token_id.clone()),
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
        status: VerificationStatus::Verified,
        links: vec![],
        is_verified: true,
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

fn asset_id_from_item(item: &NftItem, metadata: &HashMap<String, TokenMetadata>) -> Option<NFTAssetId> {
    let collection_hex = item.collection_address.as_deref()?;
    let collection = Address::from_hex_str(collection_hex).ok()?;
    if !is_verified(&collection) {
        return None;
    }
    valid_named_token_info(metadata.get(&item.address))?;
    let token = Address::from_hex_str(&item.address).ok()?;
    Some(NFTAssetId::new(Chain::Ton, &collection.to_base64_url(), &token.to_base64_url()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERIFIED_COLLECTION: &str = "EQCA14o1-VWhS2efqoh_9M1b_A9DtKTuoqfmkn83AbJzwnPi";
    const ITEM: &str = "EQCvxJy4eG8hyHBFsZ7eePxrRsUQSEUTP46abUQGAcGY6mOw";
    const NUMBERS_COLLECTION: &str = "EQAOQdwdw8kGftJCSFgOErM1mBjYPe4DBPq8-AhF6vr9si5N";
    const UNVERIFIED_COLLECTION: &str = "EQBBhhF6O-jfi1TEF1rs6pEaynEjhFrcjUCC2DfUwzJ4pRXR";

    #[test]
    fn test_map_asset_ids() {
        let response: NftItemsResponse = serde_json::from_str(include_str!("../../../testdata/ton/items.json")).unwrap();
        let asset_ids = map_asset_ids(&response);

        assert_eq!(asset_ids.len(), 1);
        let first = &asset_ids[0];
        assert_eq!(first.chain, Chain::Ton);
        assert_eq!(first.contract_address, VERIFIED_COLLECTION);
        assert_eq!(first.token_id, ITEM);
        assert_eq!(first.to_string(), format!("ton_{VERIFIED_COLLECTION}::{ITEM}"));
    }

    #[test]
    fn test_map_asset() {
        let response: NftItemsResponse = serde_json::from_str(include_str!("../../../testdata/ton/items.json")).unwrap();
        let asset_id = NFTAssetId::new(Chain::Ton, VERIFIED_COLLECTION, ITEM);
        let asset = map_asset(response, asset_id).expect("Failed to map asset");

        assert_eq!(asset.id, format!("ton_{VERIFIED_COLLECTION}::{ITEM}"));
        assert_eq!(asset.collection_id, format!("ton_{VERIFIED_COLLECTION}"));
        assert_eq!(asset.chain, Chain::Ton);
        assert_eq!(asset.token_id, ITEM);
        assert_eq!(asset.contract_address.as_deref(), Some(ITEM));
        assert_eq!(asset.name, "Resolved Item Name");
        assert_eq!(asset.token_type, NFTType::JETTON);
        assert_eq!(asset.images.preview.url, "https://example.com/resolved-item.png");
    }

    #[test]
    fn test_map_asset_rejects_unverified_collection() {
        let response: NftItemsResponse = serde_json::from_str(include_str!("../../../testdata/ton/items.json")).unwrap();
        let asset_id = NFTAssetId::new(Chain::Ton, UNVERIFIED_COLLECTION, ITEM);
        assert!(map_asset(response, asset_id).is_none());
    }

    #[test]
    fn test_map_collection() {
        let response: NftCollectionsResponse = serde_json::from_str(include_str!("../../../testdata/ton/collections.json")).unwrap();
        let collection_id = NFTCollectionId::new(Chain::Ton, NUMBERS_COLLECTION);
        let collection = map_collection(response, collection_id).expect("Failed to map collection");

        assert_eq!(collection.id, format!("ton_{NUMBERS_COLLECTION}"));
        assert_eq!(collection.chain, Chain::Ton);
        assert_eq!(collection.contract_address, NUMBERS_COLLECTION);
        assert_eq!(collection.name, "Anonymous Telegram Numbers");
        assert!(collection.is_verified);
    }

    #[test]
    fn test_map_collection_rejects_unverified() {
        let response: NftCollectionsResponse = serde_json::from_str(include_str!("../../../testdata/ton/collections_invalid.json")).unwrap();
        let collection_id = NFTCollectionId::new(Chain::Ton, UNVERIFIED_COLLECTION);
        assert!(map_collection(response, collection_id).is_none());
    }
}
