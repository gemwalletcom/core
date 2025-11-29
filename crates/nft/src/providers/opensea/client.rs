use super::model::{Collection, Contract, NftResponse, NftsResponse};
use primitives::Chain;
use std::error::Error;

pub struct OpenSeaClient {
    client: reqwest::Client,
}

impl OpenSeaClient {
    const BASE_URL: &'static str = "https://api.opensea.io";

    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    fn chain_id(chain: Chain) -> Result<&'static str, Box<dyn Error + Send + Sync>> {
        match chain {
            Chain::Ethereum => Ok("ethereum"),
            Chain::Polygon => Ok("polygon"),
            _ => Err(format!("Unsupported chain for OpenSea: {:?}", chain).into()),
        }
    }

    pub async fn get_nfts_by_account(&self, chain: Chain, account_address: &str) -> Result<NftsResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/chain/{}/account/{}/nfts", Self::BASE_URL, Self::chain_id(chain)?, account_address);
        let query = [("limit", 100)];
        Ok(self.client.get(&url).query(&query).send().await?.json().await?)
    }

    pub async fn get_collection_by_contract(&self, chain: Chain, contract_address: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        let contract = self.get_contract(chain, contract_address).await?;
        self.get_collection_by_slug(&contract.collection).await
    }

    pub async fn get_contract(&self, chain: Chain, contract_address: &str) -> Result<Contract, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/chain/{}/contract/{}", Self::BASE_URL, Self::chain_id(chain)?, contract_address);
        Ok(self.client.get(&url).send().await?.json().await?)
    }

    pub async fn get_asset_id(&self, chain: Chain, contract_address: &str, token_id: &str) -> Result<NftResponse, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "{}/api/v2/chain/{}/contract/{}/nfts/{}",
            Self::BASE_URL,
            Self::chain_id(chain)?,
            contract_address,
            token_id
        );
        Ok(self.client.get(&url).send().await?.json().await?)
    }

    pub async fn get_collection_by_slug(&self, collection_slug: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/collections/{}", Self::BASE_URL, collection_slug);
        Ok(self.client.get(&url).send().await?.json().await?)
    }
}
