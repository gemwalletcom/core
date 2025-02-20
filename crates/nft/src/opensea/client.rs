use std::error::Error;

use reqwest::header::{HeaderMap, HeaderValue};

use super::model::{Collection, Contract};

pub struct OpenSeaClient {
    client: reqwest::Client,
}

impl OpenSeaClient {
    const BASE_URL: &'static str = "https://api.opensea.io";

    pub fn new(api_key: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(&api_key).unwrap());
        OpenSeaClient {
            client: reqwest::Client::builder().default_headers(headers).build().unwrap(),
        }
    }

    pub async fn get_collection(&self, chain: &str, contract_address: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        let contract = self.get_contract(chain, contract_address).await?;
        self.get_collection_by_slug(&contract.collection).await
    }

    pub async fn get_contract(&self, chain: &str, contract_address: &str) -> Result<Contract, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/chain/{}/contract/{}", Self::BASE_URL, chain, contract_address);
        Ok(self.client.get(&url).send().await?.json::<Contract>().await?)
    }

    pub async fn get_collection_by_slug(&self, collection_slug: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/collections/{}", Self::BASE_URL, collection_slug);
        let response = self.client.get(&url).send().await?.json::<Collection>().await?;
        Ok(response)
    }
}
