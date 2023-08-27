use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use primitives::chain::Chain;
use primitives::name::{NameRecord, NameProvider};

use crate::{ens::ENSClient, ud::UDClient, sns::SNSClient, ton::TONClient};

#[async_trait]
pub trait NameClient {
    async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>>;
    fn provider() -> NameProvider;
    fn domains() -> Vec<&'static str>;
    fn chains() -> Vec<Chain>;
}

pub struct Client {
    domains_mapping: HashMap<&'static str, NameProvider>,
    ens_client: ENSClient,
    ud_client: UDClient,
    sns_client: SNSClient,
    ton_client: TONClient,
}

impl Client {
    
    pub fn new(
        ens_url: String,
        ud_url: String,
        ud_api_key: String,
        sns_url: String,
        ton_url: String,
    ) -> Self {
        let domains_mapping = Self::domains_mapping();
        let ens_client = ENSClient::new(ens_url);
        let ud_client = UDClient::new(ud_url, ud_api_key);
        let sns_client = SNSClient::new(sns_url);
        let ton_client: TONClient = TONClient::new(ton_url);

        Self {
            domains_mapping,
            ens_client,
            ud_client,
            sns_client,
            ton_client,
        }
    }

    pub async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        let name_parts = name.split('.');
        let name_prefix = name_parts.clone().last();
        
        println!("name_parts {}", name_parts.count());
        println!("name_prefix {:?}", name_prefix.clone());

        let provider = self.domains_mapping.get(name_prefix.unwrap()).unwrap();

        match provider {
            NameProvider::Ens => {
                if !ENSClient::chains().contains(&chain) {
                    return Err("not supported chain".to_string().into())
                }
                self.ens_client.resolve(name, chain).await
            }
            NameProvider::Ud => {
                if !UDClient::chains().contains(&chain) {
                    return Err("not supported chain".to_string().into())
                }
                self.ud_client.resolve(name, chain).await
            }
            NameProvider::Sns => {
                if !SNSClient::chains().contains(&chain) {
                    return Err("not supported chain".to_string().into())
                }
                self.sns_client.resolve(name, chain).await
            },
            NameProvider::Ton => {
                if !TONClient::chains().contains(&chain) {
                    return Err("not supported chain".to_string().into())
                }
                self.ton_client.resolve(name, chain).await
            }
        }
    }

    pub fn domains_mapping() -> HashMap<&'static str, NameProvider> {
        let mut result: HashMap<&'static str, NameProvider> = HashMap::new();

        for domain in ENSClient::domains() {
            result.insert(domain, NameProvider::Ens);
        }

        for domain in UDClient::domains() {
            result.insert(domain, NameProvider::Ud);
        }

        for domain in SNSClient::domains() {
            result.insert(domain, NameProvider::Sns);
        }

        for domain in TONClient::domains() {
            result.insert(domain, NameProvider::Ton);
        }

        result
    }
}