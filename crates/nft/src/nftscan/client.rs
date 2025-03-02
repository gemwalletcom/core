use evm::model::{NFTResult, Response};
use solana::NFTSolanaResult;
use ton::NFTTonResult;

use super::{
    evm::{
        self,
        model::{NFTAsset, NFTCollection},
    },
    solana, ton,
};
pub struct NFTScanClient {
    client: reqwest::Client,
    api_key: String,
}

const NFTSCAN_REST_API_URL: &str = "https://restapi.nftscan.com";
const NFTSCAN_TON_API_URL: &str = "https://tonapi.nftscan.com";
const NFTSCAN_SOLANA_API_URL: &str = "https://solanaapi.nftscan.com";

//const EVM_CHAINS: [&str; 1] = ["eth"]; //"bnb", "polygon", "arbitrum", "base"];

impl NFTScanClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.to_string(),
        }
    }
    pub async fn get_asset_id(&self, collection_id: &str, token_id: &str) -> Result<Response<NFTAsset>, reqwest::Error> {
        let url = format!("{}/api/v2/assets/{}/{}", NFTSCAN_REST_API_URL, collection_id, token_id);
        let query: [(&str, &str); 1] = [("show_attribute", "true")];
        let response = self.client.get(&url).header("X-API-KEY", &self.api_key).query(&query).send().await?;
        response.json::<Response<NFTAsset>>().await
    }

    pub async fn get_collection_id(&self, chain: &str, collection_id: &str) -> Result<Response<NFTCollection>, reqwest::Error> {
        let url = format!("{}/api/v2/collections/{}", NFTSCAN_REST_API_URL, collection_id);
        let query: [(&str, &str); 2] = [("chain", chain), ("show_attribute", "true")];
        let response = self.client.get(&url).header("X-API-KEY", &self.api_key).query(&query).send().await?;
        response.json::<Response<NFTCollection>>().await
    }

    pub async fn get_evm_nfts(&self, chains: Vec<String>, address: &str) -> Result<Response<Vec<NFTResult>>, reqwest::Error> {
        let url = format!("{}/api/v2/assets/chain/{}", NFTSCAN_REST_API_URL, address);
        let chain = chains.join(",");
        let query = [("chain", chain)];
        let response = self.client.get(&url).header("X-API-KEY", &self.api_key).query(&query).send().await?;
        response.json::<Response<Vec<NFTResult>>>().await
    }

    pub async fn get_ton_nfts(&self, address: &str) -> Result<Response<Vec<NFTTonResult>>, reqwest::Error> {
        let url = format!("{}/api/ton/account/own/all/{}", NFTSCAN_TON_API_URL, address);
        let query = [("show_attribute", "true")];
        let response = self.client.get(&url).header("X-API-KEY", &self.api_key).query(&query).send().await?;
        response.json::<Response<Vec<NFTTonResult>>>().await
    }

    pub async fn get_solana_nfts(&self, address: &str) -> Result<Response<Vec<NFTSolanaResult>>, reqwest::Error> {
        let url = format!("{}/api/sol/account/own/all/{}", NFTSCAN_SOLANA_API_URL, address);
        let query = [("show_attribute", "true")];
        let response = self.client.get(&url).header("X-API-KEY", &self.api_key).query(&query).send().await?;
        response.json::<Response<Vec<NFTSolanaResult>>>().await
    }

    pub async fn get_solana_asset_id(&self, token_id: &str) -> Result<Response<NFTAsset>, reqwest::Error> {
        let url = format!("{}/api/v2/assets/{}", NFTSCAN_REST_API_URL, token_id);
        let query: [(&str, &str); 1] = [("show_attribute", "true")];
        let response = self.client.get(&url).header("X-API-KEY", &self.api_key).query(&query).send().await?;
        response.json::<Response<NFTAsset>>().await
    }
}
