use reqwest::header::{HeaderMap, HeaderValue};

use super::model::NftResponse;

pub struct SimpleHashClient {
    client: reqwest::Client,
}

const BASE_URL: &str = "https://api.simplehash.com";
const EVM_CHAINS: [&str; 1] = ["ethereum"];

impl SimpleHashClient {
    pub fn new(api_key: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(&api_key).unwrap());
        SimpleHashClient {
            client: reqwest::Client::builder().default_headers(headers).build().unwrap(),
        }
    }

    pub async fn get_assets_evm(&self, address: &str) -> Result<NftResponse, reqwest::Error> {
        let url = format!("{}/api/v0/nfts/owners_v2", BASE_URL);
        let chains = EVM_CHAINS.join(",");
        let query = [("chains", chains), ("wallet_addresses", address.to_string()), ("limit", "50".to_string())];
        let response = self.client.get(&url).query(&query).send().await?;
        response.json::<NftResponse>().await
    }
}
