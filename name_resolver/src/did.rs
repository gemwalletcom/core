use primitives::chain::Chain;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::error::Error;

use primitives::name::{NameRecord, NameProvider};
use crate::client::NameClient;

#[derive(Debug, Deserialize, Serialize)]
pub struct Data<T> {
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Records {
    pub records: Vec<Record>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub account: String,
}

pub struct DidClient {
    api_url: String,
    client: Client,
}

impl DidClient {
    pub fn new(api_url: String) -> Self {
        let client = Client::new();
        Self {
            api_url,
            client,
        }
    }
}

#[async_trait]
impl NameClient for DidClient {
    
    fn provider() -> NameProvider {
        NameProvider::Did
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        let url = format!("{}/v2/account/records", self.api_url);
        let account = Account{ account: name.to_string() };
        let records = self.client
            .post(&url)
            .json(&account)
            .send()
            .await?
            .json::<Data<Records>>()
            .await?.data.records;

        let record = records.iter().find(|r| r.key == format!("address.{}", chain.as_slip44())).ok_or("address not found")?;

        //TODO: Use provider: self.provider
        Ok(NameRecord { name: name.to_string(), chain, address: record.value.clone(), provider: NameProvider::SpaceId })
    }

    fn domains() -> Vec<&'static str> {
        vec![
            "bit",
        ]
    }

    fn chains() -> Vec<Chain> {
        Chain::all()
    }
}
