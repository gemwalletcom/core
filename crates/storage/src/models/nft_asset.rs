use diesel::prelude::*;
use primitives::{NFTAsset, NFTImages, NFTResource};
use serde::{Deserialize, Serialize};

use crate::sql_types::{ChainRow, NftType};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::nft_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NftAssetRow {
    pub id: i32,
    pub identifier: String,
    pub collection_id: i32,
    pub chain: ChainRow,
    pub name: String,
    pub description: String,
    pub token_id: String,
    pub token_type: NftType,
    pub image_preview_url: Option<String>,
    pub image_preview_mime_type: Option<String>,
    pub resource_url: Option<String>,
    pub resource_mime_type: Option<String>,
    pub contract_address: String,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::nft_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewNftAssetRow {
    pub identifier: String,
    pub collection_id: i32,
    pub chain: ChainRow,
    pub name: String,
    pub description: String,
    pub token_id: String,
    pub token_type: NftType,
    pub image_preview_url: Option<String>,
    pub image_preview_mime_type: Option<String>,
    pub resource_url: Option<String>,
    pub resource_mime_type: Option<String>,
    pub contract_address: String,
    pub attributes: serde_json::Value,
}

impl NftAssetRow {
    pub fn as_primitive(&self, collection_identifier: String) -> NFTAsset {
        NFTAsset {
            id: self.identifier.clone(),
            collection_id: collection_identifier,
            name: self.name.clone(),
            description: Some(self.description.clone()),
            chain: self.chain.0,
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
}

impl NewNftAssetRow {
    pub fn from_primitive(primitive: NFTAsset, collection_id: i32) -> Self {
        Self {
            identifier: primitive.id.clone(),
            collection_id,
            chain: ChainRow::from(primitive.chain),
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
