use std::str::FromStr;

use diesel::prelude::*;
use primitives::{Chain, NFTImage, NFTType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::nft_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NftAsset {
    pub id: String,
    pub collection_id: String,
    pub chain: String,
    pub name: String,
    pub description: String,
    pub token_id: String,
    pub token_type: String,
    pub image_url: String,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::nft_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NftType {
    pub id: String,
}

impl NftType {
    pub fn from_primitive(primitive: primitives::NFTType) -> Self {
        Self {
            id: primitive.as_ref().to_string(),
        }
    }
}

impl NftAsset {
    pub fn as_primitive(&self) -> primitives::NFTAsset {
        primitives::NFTAsset {
            id: self.id.clone(),
            collection_id: self.collection_id.clone(),
            name: self.name.clone(),
            description: Some(self.description.clone()),
            chain: Chain::from_str(self.chain.as_str()).unwrap(),
            token_id: self.token_id.clone(),
            image: NFTImage {
                image_url: self.image_url.clone(),
                preview_image_url: self.image_url.clone(),
                original_source_url: self.image_url.clone(),
            },
            token_type: NFTType::from_str(self.token_type.as_str()).unwrap(),
            attributes: serde_json::from_value(self.attributes.clone()).unwrap_or_default(),
        }
    }

    pub fn from_primitive(primitive: primitives::NFTAsset) -> Self {
        Self {
            id: primitive.id.clone(),
            collection_id: primitive.collection_id.clone(),
            chain: primitive.chain.to_string(),
            name: primitive.name.clone(),
            description: primitive.description.unwrap_or_default(),
            token_id: primitive.token_id.clone(),
            token_type: primitive.token_type.as_ref().to_string(),
            image_url: primitive.image.image_url.clone(),
            attributes: serde_json::to_value(primitive.attributes).unwrap_or_default(),
        }
    }
}
