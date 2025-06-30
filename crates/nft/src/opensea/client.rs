use super::model::{Collection, Contract, NftResponse, NftsResponse};
use reqwest::header::{HeaderMap, HeaderValue};
use std::error::Error;

pub struct OpenSeaClient {
    client: reqwest::Client,
}

impl OpenSeaClient {
    const BASE_URL: &'static str = "https://api.opensea.io";

    pub fn new(api_key: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(api_key).unwrap());
        OpenSeaClient {
            client: reqwest::Client::builder().default_headers(headers).build().unwrap(),
        }
    }

    pub async fn get_nfts_by_account(&self, chain: &str, account_address: &str) -> Result<NftsResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/chain/{}/account/{}/nfts", Self::BASE_URL, chain, account_address);
        let query = [("limit", 100)];
        Ok(self.client.get(&url).query(&query).send().await?.json().await?)
    }

    pub async fn get_collection_id(&self, chain: &str, contract_address: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        let contract = self.get_contract(chain, contract_address).await?;
        self.get_collection_by_slug(&contract.collection).await
    }

    pub async fn get_contract(&self, chain: &str, contract_address: &str) -> Result<Contract, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/chain/{}/contract/{}", Self::BASE_URL, chain, contract_address);
        Ok(self.client.get(&url).send().await?.json().await?)
    }

    pub async fn get_asset_id(&self, chain: &str, contract_address: &str, token_id: &str) -> Result<NftResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/chain/{}/contract/{}/nfts/{}", Self::BASE_URL, chain, contract_address, token_id);
        Ok(self.client.get(&url).send().await?.json().await?)
    }

    pub async fn get_collection_by_slug(&self, collection_slug: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/collections/{}", Self::BASE_URL, collection_slug);
        Ok(self.client.get(&url).send().await?.json().await?)
    }
}
