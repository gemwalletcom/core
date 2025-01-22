use std::str::FromStr;

use diesel::prelude::*;
use primitives::{Chain, NFTImage};
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
    pub url: Option<String>,
    pub owner: Option<String>,
    pub contrtact_address: String,
    pub image_url: Option<String>,
    pub project_url: Option<String>,
    pub opensea_url: Option<String>,
    pub project_x_username: Option<String>,
    pub is_verified: bool,
    pub is_enable: bool,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::nft_collections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateNftCollectionImageUrl {
    pub id: String,
    pub image_url: Option<String>,
}

impl NftCollection {
    pub fn as_primitive(&self) -> primitives::NFTCollection {
        primitives::NFTCollection {
            id: self.id.clone(),
            name: self.name.clone(),
            description: Some(self.description.clone()),
            chain: Chain::from_str(self.chain.as_str()).unwrap(),
            contract_address: self.contrtact_address.clone(),
            image: NFTImage {
                image_url: self.image_url.clone().unwrap_or_default(),
                preview_image_url: self.image_url.clone().unwrap_or_default(),
                original_source_url: self.image_url.clone().unwrap_or_default(),
            },
            is_verified: self.is_verified,
        }
    }

    pub fn from_primitive(collection: primitives::NFTCollection) -> Self {
        NftCollection {
            id: collection.id.clone(),
            name: collection.name.clone(),
            description: collection.description.unwrap_or_default(),
            chain: collection.chain.to_string(),
            image_url: Some(collection.image.image_url.clone()),
            is_verified: collection.is_verified,
            symbol: None,
            url: None,
            owner: None,
            contrtact_address: collection.contract_address.clone(),
            project_url: None,
            opensea_url: None,
            project_x_username: None,
            is_enable: true,
        }
    }
}
