use std::str::FromStr;

use diesel::prelude::*;
use primitives::{AssetBasic, AssetId, AssetType, Chain};
use serde::{Deserialize, Serialize};
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Asset {
    pub id: String,
    pub chain: String,
    pub token_id: Option<String>,
    pub name: String,
    pub symbol: String,
    pub asset_type: String,
    pub decimals: i32,
    pub rank: i32,

    pub is_buyable: bool,
    pub is_sellable: bool,
    pub is_swappable: bool,
    pub is_stakeable: bool,

    pub staking_apr: Option<f64>,
}

impl Asset {
    pub fn as_primitive(&self) -> primitives::Asset {
        primitives::asset::Asset {
            id: AssetId {
                chain: Chain::from_str(&self.chain).unwrap(),
                token_id: self.token_id.clone(),
            },
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            asset_type: AssetType::from_str(&self.asset_type).unwrap(),
            decimals: self.decimals,
        }
    }

    pub fn as_basic_primitive(&self) -> primitives::AssetBasic {
        AssetBasic {
            asset: self.as_primitive(),
            properties: self.as_property_primitive(),
            score: self.as_score_primitive(),
        }
    }

    pub fn as_score_primitive(&self) -> primitives::AssetScore {
        primitives::AssetScore { rank: self.rank }
    }

    pub fn from_primitive(asset: primitives::Asset) -> Self {
        Self {
            id: asset.id.to_string(),
            chain: asset.id.chain.as_ref().to_string(),
            token_id: asset.id.token_id,
            name: asset.name,
            symbol: asset.symbol,
            asset_type: asset.asset_type.as_ref().to_string(),
            decimals: asset.decimals,
            rank: 0,
            is_buyable: false,
            is_sellable: false,
            is_swappable: false,
            is_stakeable: false,
            staking_apr: None,
        }
    }

    pub fn as_property_primitive(&self) -> primitives::AssetProperties {
        primitives::AssetProperties {
            is_buyable: self.is_buyable,
            is_sellable: self.is_sellable,
            is_swapable: self.is_swappable,
            is_stakeable: self.is_stakeable,
            staking_apr: self.staking_apr,
        }
    }
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::assets_links)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetLink {
    pub asset_id: String,
    pub name: String,
    pub url: String,
}

impl AssetLink {
    pub fn as_primitive(&self) -> primitives::AssetLink {
        primitives::AssetLink {
            name: self.name.clone(),
            url: self.url.clone(),
        }
    }

    pub fn from_primitive(asset_id: &str, link: primitives::AssetLink) -> Self {
        Self {
            asset_id: asset_id.to_string(),
            name: link.name.clone(),
            url: link.url.clone(),
        }
    }
}
