use serde::{Serialize, Deserialize};
use std::error::Error;
use primitives::tokenlist::TokenList;

#[derive(Debug, Clone)]
pub struct AssetsClient {
    assets_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub r#type: String,
    
    pub status: String,
    pub website: String,
    pub description: String,
    
    pub tags: Option<Vec<String>>,
    pub links: Option<Vec<AssetInfoLink>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfoLink {
    pub name: String,
    pub url: String
}

impl AssetsClient {
    pub fn new(assets_url: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            assets_url,
            client,
        }
    }

    pub async fn get_tokenlist(&self, chain: &str) -> Result<TokenList, Box<dyn Error>>  {
        let url = format!("{}/tokenlists/{}.json", self.assets_url, chain);
        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<TokenList>()
            .await?;
        Ok(response)
    }

    pub async fn get_asset_info(&self, asset_id: &str) -> Result<AssetInfo, Box<dyn Error>>  {
        let id = primitives::asset_id::AssetId::new(asset_id).unwrap();
        let url = if let Some(token_id) = id.token_id {
            format!("{}/blockchains/{}/assets/{}/info.json", self.assets_url, id.chain.as_str(), token_id)
        } else {
            format!("{}/blockchains/{}/info/info.json", self.assets_url, id.chain.as_str())
        };

        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<AssetInfo>()
            .await?;
        Ok(response)
    }
}
