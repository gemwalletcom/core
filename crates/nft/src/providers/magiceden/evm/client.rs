use super::model::{CollectionDetail, CollectionsResponse, TokenDetailResponse, TokensResponse};
use primitives::Chain;
use std::error::Error;

pub struct MagicEdenEvmClient {
    client: reqwest::Client,
}

impl MagicEdenEvmClient {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    fn chain_id(chain: Chain) -> Result<&'static str, Box<dyn Error + Send + Sync>> {
        match chain {
            Chain::SmartChain => Ok("bsc"),
            _ => Err(format!("Unsupported EVM chain for MagicEden: {:?}", chain).into()),
        }
    }

    pub async fn get_nfts_by_wallet(&self, chain: Chain, wallet_address: &str) -> Result<TokensResponse, Box<dyn Error + Send + Sync>> {
        let chain_id = Self::chain_id(chain)?;
        let url = format!("{}/v4/evm-public/assets/user-assets", super::super::BASE_URL);
        let response: TokensResponse = self
            .client
            .get(&url)
            .query(&[("chain", chain_id), ("walletAddresses[]", wallet_address)])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn fetch_collection_detail(&self, chain: Chain, collection_id: &str) -> Result<CollectionDetail, Box<dyn Error + Send + Sync>> {
        let chain_id = Self::chain_id(chain)?;
        let url = format!("{}/v4/evm-public/collections", super::super::BASE_URL);
        let body = serde_json::json!({"chain": chain_id, "collectionIds": [collection_id.to_lowercase()]});
        let response: CollectionsResponse = self.client.post(&url).json(&body).send().await?.json().await?;
        response.collections.into_iter().next().ok_or_else(|| "Collection not found".into())
    }

    pub async fn get_token(&self, chain: Chain, collection_id: &str, token_id: &str) -> Result<TokenDetailResponse, Box<dyn Error + Send + Sync>> {
        let chain_id = Self::chain_id(chain)?;
        let collection_id_lower = collection_id.to_lowercase();
        let asset_id = format!("{}:{}", collection_id_lower, token_id);
        let url = format!("{}/v4/evm-public/assets/collection-assets", super::super::BASE_URL);
        let response: TokensResponse = self
            .client
            .get(&url)
            .query(&[("chain", chain_id), ("collectionId", &collection_id_lower), ("assetIds[]", &asset_id)])
            .send()
            .await?
            .json()
            .await?;

        let token = response.assets.into_iter().next().ok_or("Token not found")?.asset;
        Ok(TokenDetailResponse { token })
    }
}
