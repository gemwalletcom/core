use super::model::{Collection, Nft};
use reqwest::header::{HeaderMap, HeaderValue};
use std::error::Error;

pub struct MagicEdenClient {
    client: reqwest::Client,
}

impl MagicEdenClient {
    const BASE_URL: &'static str = "https://api-mainnet.magiceden.dev";

    pub fn new(api_key: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_str(api_key).unwrap());
        MagicEdenClient {
            client: reqwest::Client::builder().default_headers(headers).build().unwrap(),
        }
    }

    pub async fn get_token_owner(&self, _address: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        Err("Token owner lookup not implemented - use dedicated Solana client".into())
    }

    pub async fn get_nfts_by_account(&self, address: &str) -> Result<Vec<Nft>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/v2/wallets/{address}/tokens", Self::BASE_URL))
            .send()
            .await?
            .json::<Vec<Nft>>()
            .await?)
    }

    pub async fn get_collection_id(&self, collection_id: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/collections/{collection_id}", Self::BASE_URL))
            .send()
            .await?
            .json::<Collection>()
            .await?)
    }

    pub async fn get_asset_id(&self, token_mint: &str) -> Result<Nft, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/v2/tokens/{token_mint}", Self::BASE_URL))
            .send()
            .await?
            .json::<Nft>()
            .await?)
    }
}
