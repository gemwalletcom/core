use std::str::FromStr;

use diesel::prelude::*;
use primitives::{nft::NFTImages, Chain, NFTImage, NFTImageOld, NFTType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::nft_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NftAsset {
    pub id: String,
    pub collection_id: String,
    pub contract_address: String,
    pub chain: String,
    pub name: String,
    pub description: String,
    pub token_id: String,
    pub token_type: String,
    pub image_preview_url: Option<String>,
    pub image_preview_mime_type: Option<String>,
    pub image_original_url: Option<String>,
    pub image_original_mime_type: Option<String>,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::nft_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateNftAssetImageUrl {
    pub id: String,
    pub image_preview_url: Option<String>,
    pub image_preview_mime_type: Option<String>,

    pub image_original_url: Option<String>,
    pub image_original_mime_type: Option<String>,
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
            contract_address: Some(self.contract_address.clone()),
            token_id: self.token_id.clone(),
            image: NFTImageOld {
                image_url: self.image_preview_url.clone().unwrap_or_default(),
                preview_image_url: self.image_preview_url.clone().unwrap_or_default(),
                original_source_url: self.image_preview_url.clone().unwrap_or_default(),
            },
            images: NFTImages {
                preview: NFTImage {
                    url: self.image_preview_url.clone().unwrap_or_default(),
                    mime_type: self.image_preview_mime_type.clone().unwrap_or_default(),
                },
                original: NFTImage {
                    url: self.image_original_url.clone().unwrap_or_default(),
                    mime_type: self.image_original_mime_type.clone().unwrap_or_default(),
                },
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
            contract_address: primitive.contract_address.unwrap_or_default(),
            token_id: primitive.token_id.clone(),
            token_type: primitive.token_type.as_ref().to_string(),
            image_preview_url: Some(primitive.images.preview.url.clone()),
            image_preview_mime_type: Some(primitive.images.preview.mime_type.clone()),
            image_original_url: Some(primitive.images.original.url.clone()),
            image_original_mime_type: Some(primitive.images.original.mime_type.clone()),
            attributes: serde_json::to_value(primitive.attributes).unwrap_or_default(),
        }
    }
}
