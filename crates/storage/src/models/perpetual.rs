use diesel::prelude::*;
use primitives::{
    AssetId,
    perpetual::{Perpetual as PrimitivePerpetual, PerpetualBasic},
    perpetual_provider::PerpetualProvider,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::perpetuals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Perpetual {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub asset_id: String,
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
pub struct NewPerpetualAsset {
    pub perpetual_id: String,
    pub asset_id: String,
}

impl NewPerpetualAsset {
    pub fn new(perpetual_id: String, asset_id: String) -> Self {
        Self { perpetual_id, asset_id }
    }
}

impl Perpetual {
    pub fn from_primitive(perpetual: PrimitivePerpetual) -> Self {
        Self {
            id: perpetual.id,
            name: perpetual.name,
            provider: perpetual.provider.as_ref().to_string(),
            asset_id: perpetual.asset_id.to_string(),
            identifier: perpetual.identifier,
            price: perpetual.price,
            price_percent_change_24h: perpetual.price_percent_change_24h,
            open_interest: perpetual.open_interest,
            volume_24h: perpetual.volume_24h,
            funding: perpetual.funding,
            leverage: perpetual.max_leverage.into_iter().map(|value| Some(i32::from(value))).collect(),
        }
    }

    pub fn as_primitive(&self) -> PrimitivePerpetual {
        let provider = PerpetualProvider::from_str(&self.provider).ok().unwrap();

        PrimitivePerpetual {
            id: self.id.clone(),
            name: self.name.clone(),
            provider,
            asset_id: AssetId::new(&self.asset_id).unwrap(),
            identifier: self.identifier.clone(),
            price: self.price,
            price_percent_change_24h: self.price_percent_change_24h,
            open_interest: self.open_interest,
            volume_24h: self.volume_24h,
            funding: self.funding,
            max_leverage: self.leverage.iter().filter_map(|value| value.and_then(|v| u8::try_from(v).ok())).collect(),
        }
    }

    pub fn as_basic(&self) -> PerpetualBasic {
        PerpetualBasic {
            asset_id: AssetId::new(&self.asset_id).unwrap(),
            perpetual_id: self.id.clone(),
            provider: PerpetualProvider::from_str(&self.provider).ok().unwrap(),
        }
    }
}
