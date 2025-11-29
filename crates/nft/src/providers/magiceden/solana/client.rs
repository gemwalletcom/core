use super::model::{Collection, Nft};
use std::error::Error;

pub struct MagicEdenSolanaClient {
    client: reqwest::Client,
}

impl MagicEdenSolanaClient {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn get_token_owner(&self, _address: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        Err("Token owner lookup not implemented - use dedicated Solana client".into())
    }

    pub async fn get_nfts_by_account(&self, address: &str) -> Result<Vec<Nft>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/v2/wallets/{address}/tokens", super::super::BASE_URL))
            .send()
            .await?
            .json::<Vec<Nft>>()
            .await?)
    }

    pub async fn get_collection_id(&self, collection_id: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/collections/{collection_id}", super::super::BASE_URL))
            .send()
            .await?
            .json::<Collection>()
            .await?)
    }

    pub async fn get_asset_id(&self, token_mint: &str) -> Result<Nft, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/v2/tokens/{token_mint}", super::super::BASE_URL))
            .send()
            .await?
            .json::<Nft>()
            .await?)
    }
}
