use model::{NFTResult, Response};

pub mod model;

pub struct NFTScanClient {
    client: reqwest::Client,
    api_key: String,
}

const NFTSCAN_REST_API_URL: &str = "https://restapi.nftscan.com";
const EVM_CHAINS: [&str; 5] = ["eth", "bnb", "polygon", "arbitrum", "base"];

impl NFTScanClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn get_all_evm_nfts(&self, address: &str) -> Result<Response<Vec<NFTResult>>, reqwest::Error> {
        let url = format!("{}/api/v2/assets/chain/{}", NFTSCAN_REST_API_URL, address);
        let chain = EVM_CHAINS.join(",");
        let query = [("chain", chain), ("show_attribute", "true".to_string())];
        let response = self.client.get(&url).header("X-API-KEY", &self.api_key).query(&query).send().await?;
        let nft_response = response.json::<Response<Vec<NFTResult>>>().await?;
        Ok(nft_response)
    }
}
