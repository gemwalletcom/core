use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::nft_links)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NftLink {
    pub collection_id: String,
    pub link_type: String,
    pub url: String,
}

impl NftLink {
    pub fn as_primitive(&self) -> primitives::AssetLink {
        primitives::AssetLink {
            name: self.link_type.clone(),
            url: self.url.clone(),
        }
    }

    pub fn from_primitive(collection_id: &str, link: primitives::AssetLink) -> Self {
        Self {
            collection_id: collection_id.to_string(),
            link_type: link.name.clone(),
            url: link.url.clone(),
        }
    }
}
