use std::str::FromStr;

use diesel::prelude::*;
use primitives::{AssetId, AssetLinks, AssetType, Chain};
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
            asset_type: asset.asset_type.to_string(),
            decimals: asset.decimals,
            rank: 0,
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

    pub is_buyable: bool,
    pub is_sellable: bool,
    pub is_swappable: bool,
    pub is_stakeable: bool,
}

impl AssetDetail {
    pub fn as_primitive(&self) -> primitives::AssetDetails {
        primitives::AssetDetails {
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
            },
            is_buyable: self.is_buyable,
            is_sellable: self.is_sellable,
            is_swapable: self.is_swappable,
            is_stakeable: self.is_stakeable,
        }
    }

    pub fn from_primitive(asset_id: &str, value: primitives::AssetDetails) -> AssetDetail {
        AssetDetail {
            asset_id: asset_id.to_string(),
            homepage: value.links.homepage,
            explorer: value.links.explorer,
            twitter: value.links.twitter,
            telegram: value.links.telegram,
            github: value.links.github,
            youtube: value.links.youtube,
            facebook: value.links.facebook,
            reddit: value.links.reddit,
            coingecko: value.links.coingecko,
            coinmarketcap: value.links.coinmarketcap,
            discord: value.links.discord,
            is_buyable: value.is_buyable,
            is_sellable: value.is_sellable,
            is_swappable: value.is_swapable,
            is_stakeable: value.is_stakeable,
        }
    }
}
