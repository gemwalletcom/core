use gem_ton::models::{NftAttribute, NftCollection, NftContent, NftItem};
use primitives::{AssetLink, Chain, LinkType, NFTAsset, NFTAssetId, NFTAttribute, NFTCollectionId, NFTImages, NFTResource, NFTType, VerificationStatus};

pub fn map_asset_ids(items: &[NftItem]) -> Vec<NFTAssetId> {
    items.iter().filter_map(asset_id_from_item).collect()
}

pub fn map_nft_assets(items: Vec<NftItem>) -> Vec<NFTAsset> {
    items.into_iter().filter_map(|item| asset_id_from_item(&item).and_then(|id| map_asset(item, id))).collect()
}

pub fn map_asset(item: NftItem, asset_id: NFTAssetId) -> Option<NFTAsset> {
    let content = item.content.unwrap_or_default();
    let image = content.image.clone().unwrap_or_default();
    let attributes = content.attributes.iter().filter_map(attribute_from).collect();
    let index = item.index.unwrap_or_default();
    Some(NFTAsset {
        id: asset_id.to_string(),
        collection_id: asset_id.get_collection_id().id(),
        contract_address: Some(item.address),
        token_id: asset_id.token_id.clone(),
        token_type: NFTType::JETTON,
        name: content
            .name
            .unwrap_or_else(|| if index.is_empty() { asset_id.token_id.clone() } else { format!("#{}", index) }),
        description: content.description,
        chain: asset_id.chain,
        resource: NFTResource::from_url(&image),
        images: NFTImages {
            preview: NFTResource::from_url(&image),
        },
        attributes,
    })
}

pub fn map_collection(collection: NftCollection, collection_id: NFTCollectionId) -> primitives::NFTCollection {
    let content = collection.collection_content.unwrap_or_default();
    let image = content.image.clone().unwrap_or_default();
    let links = links_from(&content);
    primitives::NFTCollection {
        id: collection_id.id(),
        name: content.name.unwrap_or_else(|| collection.address.clone()),
        symbol: None,
        description: content.description,
        chain: collection_id.chain,
        contract_address: collection.address,
        images: NFTImages {
            preview: NFTResource::from_url(&image),
        },
        status: VerificationStatus::Unverified,
        links,
        is_verified: false,
    }
}

fn asset_id_from_item(item: &NftItem) -> Option<NFTAssetId> {
    let collection = item.collection_address.as_deref()?;
    Some(NFTAssetId::new(Chain::Ton, collection, &item.address))
}

fn attribute_from(attribute: &NftAttribute) -> Option<NFTAttribute> {
    let name = attribute.trait_type.clone()?;
    let value = match attribute.value.as_ref()? {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    Some(NFTAttribute { name, value, percentage: None })
}

fn links_from(content: &NftContent) -> Vec<AssetLink> {
    let mut links = vec![];
    if let Some(url) = content.external_url.as_deref().or(content.external_link.as_deref())
        && !url.is_empty()
    {
        links.push(AssetLink::new(url, LinkType::Website));
    }
    if let Some(socials) = &content.social_links {
        for url in socials {
            if !url.is_empty() {
                links.push(AssetLink::new(url, LinkType::Website));
            }
        }
    }
    links
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_ton::models::{NftCollection, NftItem, NftItemsResponse};

    const COLLECTION_ADDRESS: &str = "0:1DC6211C5F90A1C08F185E8EE369C1DF712EAC74C24ECEEFD68C90DC51A94717";
    const ITEM_ADDRESS: &str = "0:AFC49CB8786F21C87045B19EDE78FC6B46C5104845133F8E9A6D440601C198EA";

    #[test]
    fn test_map_asset_ids() {
        let response: NftItemsResponse = serde_json::from_str(include_str!("../../../testdata/ton/items.json")).unwrap();
        let asset_ids = map_asset_ids(&response.nft_items);

        assert_eq!(asset_ids.len(), 1);
        let first = &asset_ids[0];
        assert_eq!(first.chain, Chain::Ton);
        assert_eq!(first.contract_address, COLLECTION_ADDRESS);
        assert_eq!(first.token_id, ITEM_ADDRESS);
    }

    #[test]
    fn test_map_asset() {
        let item: NftItem = serde_json::from_str(include_str!("../../../testdata/ton/item.json")).unwrap();
        let asset_id = NFTAssetId::new(Chain::Ton, COLLECTION_ADDRESS, ITEM_ADDRESS);
        let asset = map_asset(item, asset_id).expect("Failed to map asset");

        assert_eq!(asset.chain, Chain::Ton);
        assert_eq!(asset.token_id, ITEM_ADDRESS);
        assert_eq!(asset.contract_address.as_deref(), Some(ITEM_ADDRESS));
        assert_eq!(asset.collection_id, format!("ton_{}", COLLECTION_ADDRESS));
        assert_eq!(asset.name, "DOGS #42");
        assert_eq!(asset.token_type, NFTType::JETTON);
        assert_eq!(asset.images.preview.url, "https://example.com/dogs/42.png");

        let background = asset.attributes.iter().find(|a| a.name == "Background").unwrap();
        assert_eq!(background.value, "Blue");
        let rarity = asset.attributes.iter().find(|a| a.name == "Rarity").unwrap();
        assert_eq!(rarity.value, "5");
    }

    #[test]
    fn test_map_collection() {
        let collection: NftCollection = serde_json::from_str(include_str!("../../../testdata/ton/collection.json")).unwrap();
        let collection_id = NFTCollectionId::new(Chain::Ton, COLLECTION_ADDRESS);
        let nft_collection = map_collection(collection, collection_id);

        assert_eq!(nft_collection.chain, Chain::Ton);
        assert_eq!(nft_collection.contract_address, COLLECTION_ADDRESS);
        assert_eq!(nft_collection.name, "DOGS");
        assert_eq!(nft_collection.description.as_deref(), Some("DOGS NFT collection on TON"));
        assert_eq!(nft_collection.images.preview.url, "https://example.com/dogs/cover.png");
        assert_eq!(nft_collection.status, VerificationStatus::Unverified);
        assert!(nft_collection.links.iter().any(|l| l.url == "https://dogs.com"));
        assert!(nft_collection.links.iter().any(|l| l.url == "https://twitter.com/dogs"));
    }
}
