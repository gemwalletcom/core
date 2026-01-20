use std::str::FromStr;

use diesel::prelude::*;
use primitives::AssetLink;
use serde::{Deserialize, Serialize};

use crate::sql_types::LinkType;

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::nft_collections_links)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NftLinkRow {
    pub collection_id: String,
    pub link_type: LinkType,
    pub url: String,
}

impl NftLinkRow {
    pub fn as_primitive(&self) -> AssetLink {
        AssetLink {
            name: self.link_type.as_ref().to_string(),
            url: self.url.clone(),
        }
    }

    pub fn from_primitive(collection_id: &str, link: AssetLink) -> Self {
        Self {
            collection_id: collection_id.to_string(),
            link_type: primitives::LinkType::from_str(&link.name).unwrap().into(),
            url: link.url.clone(),
        }
    }
}
