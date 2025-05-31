use std::str::FromStr;

use diesel::prelude::*;
use primitives::{AssetLink, Chain, NFTImages, NFTResource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::nft_collections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NftCollection {
    pub id: String,
    pub chain: String,
    pub name: String,
    pub description: String,
    pub symbol: Option<String>,
    pub owner: Option<String>,
    pub contract_address: String,
    pub image_preview_url: Option<String>,
    pub image_preview_mime_type: Option<String>,
    pub is_verified: bool,
    pub is_enabled: bool,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::nft_collections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateNftCollectionImageUrl {
    pub id: String,
    pub image_preview_url: Option<String>,
    pub image_preview_mime_type: Option<String>,
}

impl NftCollection {
    pub fn as_primitive(&self, links: Vec<AssetLink>) -> primitives::NFTCollection {
        primitives::NFTCollection {
            id: self.id.clone(),
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            description: Some(self.description.clone()),
            chain: Chain::from_str(self.chain.as_str()).unwrap(),
            contract_address: self.contract_address.clone(),
            images: NFTImages {
                preview: NFTResource {
                    url: self.image_preview_url.clone().unwrap_or_default(),
                    mime_type: self.image_preview_mime_type.clone().unwrap_or_default(),
                },
            },
            is_verified: self.is_verified,
            links,
        }
    }

    pub fn from_primitive(collection: primitives::NFTCollection) -> Self {
        NftCollection {
            id: collection.id.clone(),
            name: collection.name.clone(),
            description: collection.description.unwrap_or_default(),
            chain: collection.chain.to_string(),
            image_preview_url: Some(collection.images.preview.url.clone()),
            image_preview_mime_type: Some(collection.images.preview.mime_type.clone()),
            is_verified: collection.is_verified,
            symbol: collection.symbol,
            owner: None,
            contract_address: collection.contract_address.clone(),
            is_enabled: true,
        }
    }
}
