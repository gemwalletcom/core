use std::str::FromStr;

use diesel::prelude::*;
use primitives::{AssetBasic, AssetId, AssetProperties, AssetScore, AssetType, Chain};
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

    pub is_enabled: bool,
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
            links: vec![],
        }
    }

    pub fn as_score_primitive(&self) -> primitives::AssetScore {
        primitives::AssetScore { rank: self.rank }
    }

    pub fn from_primitive_default(asset: primitives::Asset) -> Self {
        Self::from_primitive(asset.clone(), AssetScore::default(), AssetProperties::default(asset.chain()))
    }

    pub fn from_primitive(asset: primitives::Asset, score: AssetScore, properties: primitives::AssetProperties) -> Self {
        Self {
            id: asset.id.to_string(),
            chain: asset.id.chain.as_ref().to_string(),
            token_id: asset.id.token_id,
            name: asset.name,
            symbol: asset.symbol,
            asset_type: asset.asset_type.as_ref().to_string(),
            decimals: asset.decimals,
            rank: score.rank,
            is_enabled: properties.is_enabled,
            is_buyable: properties.is_buyable,
            is_sellable: properties.is_sellable,
            is_swappable: properties.is_swapable,
            is_stakeable: properties.is_stakeable,
            staking_apr: properties.staking_apr,
        }
    }

    pub fn as_property_primitive(&self) -> primitives::AssetProperties {
        primitives::AssetProperties {
            is_enabled: self.is_enabled,
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
    pub link_type: String,
    pub url: String,
}

impl AssetLink {
    pub fn as_primitive(&self) -> primitives::AssetLink {
        primitives::AssetLink {
            name: self.link_type.clone(),
            url: self.url.clone(),
        }
    }

    pub fn from_primitive(asset_id: &str, link: primitives::AssetLink) -> Self {
        Self {
            asset_id: asset_id.to_string(),
            link_type: link.name.clone(),
            url: link.url.clone(),
        }
    }
}
