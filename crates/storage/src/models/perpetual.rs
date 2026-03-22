use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{
    AssetId as PrimitiveAssetId,
    perpetual::{Perpetual as PrimitivePerpetual, PerpetualBasic},
};
use serde::{Deserialize, Serialize};

use crate::sql_types::{AssetId, PerpetualProviderRow};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::perpetuals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PerpetualRow {
    pub id: String,
    pub name: String,
    pub provider: PerpetualProviderRow,
    pub asset_id: AssetId,
    pub identifier: String,
    pub price: f64,
    pub price_percent_change_24h: f64,
    pub open_interest: f64,
    pub volume_24h: f64,
    pub funding: f64,
    pub leverage: Vec<Option<i32>>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::perpetuals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPerpetualRow {
    pub id: String,
    pub name: String,
    pub provider: PerpetualProviderRow,
    pub asset_id: AssetId,
    pub identifier: String,
    pub price: f64,
    pub price_percent_change_24h: f64,
    pub open_interest: f64,
    pub volume_24h: f64,
    pub funding: f64,
    pub leverage: Vec<Option<i32>>,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::perpetuals_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPerpetualAssetRow {
    pub perpetual_id: String,
    pub asset_id: AssetId,
}

impl NewPerpetualAssetRow {
    pub fn new(perpetual_id: String, asset_id: PrimitiveAssetId) -> Self {
        Self {
            perpetual_id,
            asset_id: asset_id.into(),
        }
    }
}

impl NewPerpetualRow {
    pub fn from_primitive(perpetual: PrimitivePerpetual) -> Self {
        Self {
            id: perpetual.id,
            name: perpetual.name,
            provider: perpetual.provider.into(),
            asset_id: perpetual.asset_id.into(),
            identifier: perpetual.identifier,
            price: perpetual.price,
            price_percent_change_24h: perpetual.price_percent_change_24h,
            open_interest: perpetual.open_interest,
            volume_24h: perpetual.volume_24h,
            funding: perpetual.funding,
            leverage: vec![Some(i32::from(perpetual.max_leverage))],
        }
    }
}

impl PerpetualRow {
    pub fn as_primitive(&self) -> PrimitivePerpetual {
        PrimitivePerpetual {
            id: self.id.clone(),
            name: self.name.clone(),
            provider: self.provider.0.clone(),
            asset_id: self.asset_id.0.clone(),
            identifier: self.identifier.clone(),
            price: self.price,
            price_percent_change_24h: self.price_percent_change_24h,
            open_interest: self.open_interest,
            volume_24h: self.volume_24h,
            funding: self.funding,
            max_leverage: self.leverage.first().and_then(|v| v.and_then(|i| u8::try_from(i).ok())).unwrap_or(1),
            only_isolated: false,
        }
    }

    pub fn as_basic(&self) -> PerpetualBasic {
        PerpetualBasic {
            asset_id: self.asset_id.0.clone(),
            perpetual_id: self.id.clone(),
            provider: self.provider.0.clone(),
        }
    }
}
