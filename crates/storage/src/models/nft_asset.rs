use std::str::FromStr;

use diesel::prelude::*;
use primitives::{Chain, NFTAsset, NFTImages, NFTResource};
use serde::{Deserialize, Serialize};

use crate::sql_types::NftType;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::nft_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NftAssetRow {
    pub id: String,
    pub collection_id: String,
    pub contract_address: String,
    pub chain: String,
    pub name: String,
    pub description: String,
    pub token_id: String,
    pub token_type: NftType,
    pub image_preview_url: Option<String>,
    pub image_preview_mime_type: Option<String>,
    pub resource_url: Option<String>,
    pub resource_mime_type: Option<String>,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::nft_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateNftAssetImageUrlRow {
    pub id: String,
    pub image_preview_url: Option<String>,
    pub image_preview_mime_type: Option<String>,
}

impl NftAssetRow {
    pub fn as_primitive(&self) -> NFTAsset {
        NFTAsset {
            id: self.id.clone(),
            collection_id: self.collection_id.clone(),
            name: self.name.clone(),
            description: Some(self.description.clone()),
            chain: Chain::from_str(self.chain.as_str()).unwrap(),
            contract_address: Some(self.contract_address.clone()),
            token_id: self.token_id.clone(),
            resource: NFTResource {
                url: self.resource_url.clone().unwrap_or_default(),
                mime_type: self.resource_mime_type.clone().unwrap_or_default(),
            },
            images: NFTImages {
                preview: NFTResource {
                    url: self.image_preview_url.clone().unwrap_or_default(),
                    mime_type: self.image_preview_mime_type.clone().unwrap_or_default(),
                },
            },
            token_type: self.token_type.0.clone(),
            attributes: serde_json::from_value(self.attributes.clone()).unwrap_or_default(),
        }
    }

    pub fn from_primitive(primitive: NFTAsset) -> Self {
        Self {
            id: primitive.id.clone(),
            collection_id: primitive.collection_id.clone(),
            chain: primitive.chain.to_string(),
            name: primitive.name.clone(),
            description: primitive.description.unwrap_or_default(),
            contract_address: primitive.contract_address.unwrap_or_default(),
            token_id: primitive.token_id.clone(),
            token_type: primitive.token_type.into(),
            image_preview_url: Some(primitive.images.preview.url.clone()),
            image_preview_mime_type: Some(primitive.images.preview.mime_type.clone()),
            resource_url: Some(primitive.resource.url.clone()),
            resource_mime_type: Some(primitive.resource.mime_type.clone()),
            attributes: serde_json::to_value(primitive.attributes).unwrap_or_default(),
        }
    }
}
