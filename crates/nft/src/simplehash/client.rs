use reqwest::header::{HeaderMap, HeaderValue};

use super::model::NftResponse;

pub struct SimpleHashClient {
    client: reqwest::Client,
}

const BASE_URL: &str = "https://api.simplehash.com";
pub(crate) const SIMPLEHASH_EVM_CHAINS: [&str; 1] = ["ethereum"];
pub(crate) const SIMPLEHASH_SOLANA_CHAIN: [&str; 1] = ["solana"];

impl SimpleHashClient {
    pub fn new(api_key: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(&api_key).unwrap());
        SimpleHashClient {
            client: reqwest::Client::builder().default_headers(headers).build().unwrap(),
        }
    }

    pub async fn get_assets_all(&self, address: &str, chains: Vec<&str>, pages_limit: i32) -> Result<NftResponse, reqwest::Error> {
        let mut cursor = None;
        let mut all_assets = Vec::new();
        let mut current_page = 0;

        loop {
            let response = self.get_assets(address, chains.clone(), cursor.clone()).await?;
            all_assets.extend(response.nfts);

            if let Some(next_cursor) = response.next_cursor {
                cursor = Some(next_cursor);
                current_page += 1;
            } else {
                break;
            }
            if current_page >= pages_limit {
                break;
            }
        }

        Ok(NftResponse {
            nfts: all_assets,
            next_cursor: None,
        })
    }

    pub async fn get_assets(&self, address: &str, chains: Vec<&str>, cursor: Option<String>) -> Result<NftResponse, reqwest::Error> {
        let url = format!("{}/api/v0/nfts/owners_v2", BASE_URL);
        let chains = chains.join(",");
        let mut query = vec![
            ("chains", chains),
            ("wallet_addresses", address.to_string()),
            ("limit", "50".to_string()),
            ("include_attribute_percentages", "1".to_string()),
            ("spam_score__lte", "50".to_string()),
        ];
        if let Some(cursor) = cursor {
            query.push(("cursor", cursor));
        }
        let response = self.client.get(&url).query(&query).send().await?;
        response.json::<NftResponse>().await
    }
}
