use async_trait::async_trait;
use primitives::chain::Chain;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

use crate::client::NameClient;
use primitives::name::{NameProvider};

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolveDomain {
    pub records: HashMap<String, String>,
}

pub struct UDClient {
    api_url: String,
    api_key: String,
    client: Client,
}

impl UDClient {
    pub fn new(api_url: String, api_key: String) -> Self {
        let client = Client::new();
        Self {
            api_url,
            api_key,
            client,
        }
    }

    fn map(&self, chain: Chain, records: HashMap<String, String>) -> Option<String> {
        match chain {
            Chain::Bitcoin => records.get("crypto.BTC.address").cloned(),
            Chain::Solana => records.get("crypto.SOL.address").cloned(),
            Chain::Ethereum => records.get("crypto.ETH.address").cloned(),
            Chain::Polygon => records.get("crypto.MATIC.version.MATIC.address").cloned(),
            Chain::Base => records.get("crypto.ETH.address").cloned(),
            Chain::Arbitrum => records.get("crypto.ETH.address").cloned(),
            Chain::Optimism => records.get("crypto.ETH.address").cloned(),
            Chain::AvalancheC => records.get("crypto.ETH.address").cloned(),
            Chain::Tron => records.get("crypto.TRX.address").cloned(),
            Chain::Cosmos => records.get("crypto.ATOM.address").cloned(),
            Chain::Doge => records.get("crypto.DOGE.address").cloned(),
            Chain::Binance => records.get("crypto.BNB.version.BEP2.address").cloned(),
            Chain::SmartChain => records.get("crypto.BNB.version.BEP20.address").cloned(),
            Chain::Aptos => records.get("crypto.APT.address").cloned(),
            _ => None,
        }
    }
}

#[async_trait]
impl NameClient for UDClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Ud
    }

    async fn resolve(
        &self,
        name: &str,
        chain: Chain,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/resolve/domains/{}", self.api_url, name);
        let response = self
            .client
            .get(&url)
            .bearer_auth(self.api_key.clone())
            .send()
            .await?
            .json::<ResolveDomain>()
            .await?;
        let records = response.records;

        let address = self.map(chain, records);
        match address {
            None => Err("address not found".into()),
            Some(address) => Ok(address),
        }
    }

    fn domains(&self) -> Vec<&'static str> {
        // https://api.unstoppabledomains.com/resolve/supported_tlds
        vec![
            "crypto",
            "bitcoin",
            "blockchain",
            "dao",
            "nft",
            "888",
            "wallet",
            "x",
            "klever",
            "hi",
            "kresus",
            "polygon",
            "anime",
            "manga",
            "binanceus",
            "zil",
        ]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![
            Chain::Bitcoin,
            Chain::Ethereum,
            Chain::Solana,
            Chain::Tron,
            Chain::Cosmos,
            Chain::Doge,
            Chain::Binance,
            Chain::SmartChain,
            Chain::Polygon,
            Chain::Optimism,
            Chain::AvalancheC,
            Chain::Aptos,
        ]
    }
}
