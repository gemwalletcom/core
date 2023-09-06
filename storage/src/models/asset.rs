use diesel::prelude::*;
use primitives::{chain::Chain, asset_type::AssetType, asset_id::AssetId};
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
}

impl Asset {
    pub fn as_primitive(&self) -> primitives::asset::Asset {
        primitives::asset::Asset{
            id: AssetId {chain: Chain::from_str(&self.chain).unwrap(), token_id: self.token_id.clone() },
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            asset_type: AssetType::from_str(&self.asset_type).unwrap(),
            decimals: self.decimals
        }
    }

    pub fn from_primitive(asset: primitives::asset::Asset) -> Self {
        Self {
            id: asset.id.to_string(),
            chain: asset.id.chain.to_string(),
            token_id: asset.id.token_id,
            name: asset.name,
            symbol: asset.symbol,
            asset_type: asset.asset_type.to_string(),
            decimals: asset.decimals
        }
    }
}