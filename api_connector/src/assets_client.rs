use std::error::Error;
use primitives::tokenlist::TokenList;
use primitives::asset_info::AssetInfo;

#[derive(Debug, Clone)]
pub struct AssetsClient {
    assets_url: String,
    client: reqwest::Client,
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
        println!("url: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<AssetInfo>()
            .await?;
        Ok(response)
    }
}
