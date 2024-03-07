use crate::codec::Codec;
use crate::{client::NameClient, ton_codec};
use async_trait::async_trait;
use primitives::{Chain, NameProvider};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct TONClient {
    url: String,
    client: Client,
}

impl TONClient {
    pub fn new(url: String) -> Self {
        let client = Client::new();
        Self { url, client }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolveWallet {
    pub address: String,
    pub is_wallet: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolveResponse {
    pub wallet: ResolveWallet,
}

#[async_trait]
impl NameClient for TONClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Ton
    }

    async fn resolve(
        &self,
        name: &str,
        _chain: Chain,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/dns/{}/resolve", self.url, name);
        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<ResolveResponse>()
            .await?;
        // always encode as Bounceable address
        let encoded = ton_codec::TonCodec::encode(response.wallet.address.as_bytes().to_vec());
        Ok(encoded)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["ton"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Ton]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding() {
        let raw = "0:8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78";
        let address = ton_codec::TonCodec::encode(raw.as_bytes().to_vec());

        assert_eq!(address, "EQCOh0t62bvrv8SIELiTnJj1BYAkbxmYIEDbyyU8TD2veND8");
    }
}
