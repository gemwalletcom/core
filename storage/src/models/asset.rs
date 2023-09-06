use diesel::prelude::*;
use primitives::{tokenlist::TokenListAsset, chain::Chain, asset_type::AssetType, asset_id::AssetId};
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
    pub fn as_primitive(&self) -> TokenListAsset {
        TokenListAsset{
            chain: Chain::new(&self.chain).unwrap(),
            token_id: self.token_id.clone().unwrap_or_default(),
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            asset_type: AssetType::from_str(&self.asset_type).unwrap(),
            decimals: self.decimals
        }
    }

    pub fn from_primitive(asset: TokenListAsset) -> Self {
        Self {
            id: AssetId {chain: asset.chain, token_id: asset.token_id.clone().into() }.to_string(),
            chain: asset.chain.as_str().to_string(),
            token_id: Some(asset.token_id),
            name: asset.name,
            symbol: asset.symbol,
            asset_type: asset.asset_type.as_str().to_string(),
            decimals: asset.decimals
        }
    }
}