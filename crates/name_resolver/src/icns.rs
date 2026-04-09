use async_trait::async_trait;

use gem_encoding::encode_base64;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, sync::LazyLock};

use crate::client::NameClient;
use crate::model::NameQuery;
use primitives::{Chain, NameProvider};

#[derive(Debug, Deserialize, Serialize)]
pub struct Data<T> {
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    pub bech32_address: String,
}

const RESOLVER: &str = "osmo1xk0s8xgktn9x5vwcgtjdxqzadg88fgn33p8u9cnpdxwemvxscvast52cdd";

// https://github.com/satoshilabs/slips/blob/master/slip-0173.md
static DOMAIN_MAP: LazyLock<HashMap<&'static str, Chain>> =
    LazyLock::new(|| HashMap::from([("cosmos", Chain::Cosmos), ("osmo", Chain::Osmosis), ("celestia", Chain::Celestia), ("sei", Chain::Sei)]));

pub struct IcnsClient {
    api_url: String,
    client: Client,
}

impl IcnsClient {
    pub fn new(api_url: String) -> Self {
        let client = Client::new();

        Self { api_url, client }
    }
}

#[async_trait]
impl NameClient for IcnsClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Icns
    }

    async fn resolve(&self, query: &NameQuery, chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        let suffix = &query.suffix;
        if !DOMAIN_MAP.contains_key(suffix.as_str()) {
            return Err(format!("unsupported domain: {suffix}").into());
        }

        // chain type should match domain type
        let suffix_chain = DOMAIN_MAP.get(suffix.as_str()).unwrap();
        if *suffix_chain != chain {
            return Err(format!("domain: {suffix} doesn't match chain: {chain}").into());
        }

        let rpc_query = serde_json::json!({
            "address_by_icns": {
              "icns": query.domain,
            },
        });

        let b64 = encode_base64(rpc_query.to_string().as_bytes());
        let url = format!("{}/cosmwasm/wasm/v1/contract/{}/smart/{}", self.api_url, RESOLVER, b64);
        let address = self.client.get(&url).send().await?.json::<Data<Record>>().await?.data.bech32_address;

        Ok(address)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec![] // DOMAIN_MAP.keys().cloned().collect()
    }

    fn chains(&self) -> Vec<Chain> {
        DOMAIN_MAP.values().cloned().collect()
    }
}
