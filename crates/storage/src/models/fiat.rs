use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::fiat_rates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatRate {
    pub symbol: String,
    pub name: String,
    pub rate: f64,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatAsset {
    pub asset_id: String,
    pub provider: String,
    pub symbol: String,
    pub network: Option<String>,
    pub enabled: bool,
}

impl FiatAsset {
    pub fn from_primitive(asset: primitives::FiatAsset) -> Self {
        Self {
            asset_id: asset.asset_id.to_string(),
            provider: asset.provider,
            symbol: asset.symbol,
            network: asset.network,
            enabled: asset.enabled,
        }
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::fiat_providers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatProvider {
    pub id: String,
    pub name: String,
    pub enabled: bool,
}
