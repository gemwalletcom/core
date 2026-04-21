use std::str::FromStr;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{Asset, AssetBasic, AssetId as PrimitiveAssetId, AssetLink, AssetProperties, AssetScore};
use serde::{Deserialize, Serialize};

use crate::sql_types::{AssetId, AssetType, ChainRow, LinkType};

#[derive(Debug, Queryable, Selectable, Identifiable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetRow {
    pub id: String,
    pub chain: ChainRow,
    pub token_id: Option<String>,
    pub name: String,
    pub symbol: String,
    pub asset_type: AssetType,
    pub decimals: i32,
    pub rank: i32,

    pub is_enabled: bool,
    pub is_buyable: bool,
    pub is_sellable: bool,
    pub is_swappable: bool,
    pub is_stakeable: bool,
    pub staking_apr: Option<f64>,
    pub is_earnable: bool,
    pub earn_apr: Option<f64>,
    pub has_image: bool,
    pub has_price: bool,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,

    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewAssetRow {
    pub id: String,
    pub chain: ChainRow,
    pub token_id: Option<String>,
    pub name: String,
    pub symbol: String,
    pub asset_type: AssetType,
    pub decimals: i32,
    pub rank: i32,

    pub is_enabled: bool,
    pub is_buyable: bool,
    pub is_sellable: bool,
    pub is_swappable: bool,
    pub is_stakeable: bool,
    pub staking_apr: Option<f64>,
    pub is_earnable: bool,
    pub earn_apr: Option<f64>,
    pub has_image: bool,
    pub has_price: bool,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
}

impl NewAssetRow {
    pub fn from_primitive_default(asset: Asset) -> Self {
        Self::from_primitive(asset.clone(), AssetScore::default(), AssetProperties::default(asset.id))
    }

    pub fn from_primitive(asset: Asset, score: AssetScore, properties: AssetProperties) -> Self {
        Self {
            id: asset.id.to_string(),
            chain: ChainRow::from(asset.id.chain),
            token_id: asset.id.token_id,
            name: asset.name,
            symbol: asset.symbol,
            asset_type: asset.asset_type.into(),
            decimals: asset.decimals,
            rank: score.rank,
            is_enabled: properties.is_enabled,
            is_buyable: properties.is_buyable,
            is_sellable: properties.is_sellable,
            is_swappable: properties.is_swapable,
            is_stakeable: properties.is_stakeable,
            staking_apr: properties.staking_apr,
            is_earnable: properties.is_earnable,
            earn_apr: properties.earn_apr,
            has_image: properties.has_image,
            has_price: properties.has_price,
            circulating_supply: None,
            total_supply: None,
            max_supply: None,
        }
    }
}

impl AssetRow {
    pub fn as_asset_id(&self) -> PrimitiveAssetId {
        PrimitiveAssetId {
            chain: self.chain.0,
            token_id: self.token_id.clone(),
        }
    }

    pub fn as_primitive(&self) -> Asset {
        Asset::new(self.as_asset_id(), self.name.clone(), self.symbol.clone(), self.decimals, self.asset_type.0.clone())
    }

    pub fn as_basic_primitive(&self) -> AssetBasic {
        AssetBasic::new(self.as_primitive(), self.as_property_primitive(), self.as_score_primitive())
    }

    pub fn as_score_primitive(&self) -> AssetScore {
        AssetScore::new(self.rank)
    }

    pub fn as_property_primitive(&self) -> AssetProperties {
        AssetProperties {
            is_enabled: self.is_enabled,
            is_buyable: self.is_buyable,
            is_sellable: self.is_sellable,
            is_swapable: self.is_swappable,
            is_stakeable: self.is_stakeable,
            staking_apr: self.staking_apr,
            is_earnable: self.is_earnable,
            earn_apr: self.earn_apr,
            has_image: self.has_image,
            has_price: self.has_price,
        }
    }
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::assets_links)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetLinkRow {
    pub asset_id: AssetId,
    pub link_type: LinkType,
    pub url: String,
}

impl AssetLinkRow {
    pub fn as_primitive(&self) -> AssetLink {
        AssetLink {
            name: self.link_type.as_ref().to_string(),
            url: self.url.clone(),
        }
    }

    pub fn from_primitive(asset_id: &PrimitiveAssetId, link: AssetLink) -> Self {
        Self {
            asset_id: asset_id.into(),
            link_type: primitives::LinkType::from_str(&link.name).unwrap().into(),
            url: link.url.clone(),
        }
    }
}
