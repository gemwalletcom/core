use diesel::prelude::*;
use primitives::{Chain, AssetType, AssetId, AssetLinks};
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

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::assets_details)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetDetail {
    pub asset_id: String,
    // links
    pub homepage: Option<String>,
    pub explorer: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub github: Option<String>,
    pub youtube: Option<String>,
    pub facebook: Option<String>,
    pub reddit: Option<String>,
    pub coingecko: Option<String>,
    pub coinmarketcap: Option<String>,
    pub discord: Option<String>,
}

impl AssetDetail {
    pub fn as_primitive(&self) -> primitives::AssetDetails {
        primitives::AssetDetails{
            links: AssetLinks {
                homepage: self.homepage.clone(),
                explorer: self.explorer.clone(),
                twitter: self.twitter.clone(),
                telegram: self.telegram.clone(),
                github: self.github.clone(),
                youtube: self.youtube.clone(),
                facebook: self.facebook.clone(),
                reddit: self.reddit.clone(),
                coingecko: self.coingecko.clone(),
                coinmarketcap: self.coinmarketcap.clone(),
                discord: self.discord.clone(),
            }
        }
    }
}