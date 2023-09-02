use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use primitives::chain::Chain;
use primitives::name::{NameRecord, NameProvider};

use crate::{ens::ENSClient, ud::UDClient, sns::SNSClient, ton::TONClient, tree::TreeClient, spaceid::SpaceIdClient};

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
    tree_client: TreeClient,
    spaceid_client: SpaceIdClient,
}

impl Client {
    
    pub fn new(
        ens_url: String,
        ud_url: String,
        ud_api_key: String,
        sns_url: String,
        ton_url: String,
        tree_api_url: String,
        space_api_url: String,
    ) -> Self {
        let domains_mapping = Self::domains_mapping();
        let ens_client = ENSClient::new(ens_url);
        let ud_client = UDClient::new(ud_url, ud_api_key);
        let sns_client = SNSClient::new(sns_url);
        let ton_client: TONClient = TONClient::new(ton_url);
        let tree_client: TreeClient = TreeClient::new(tree_api_url);
        let spaceid_client: SpaceIdClient = SpaceIdClient::new(space_api_url);

        Self {
            domains_mapping,
            ens_client,
            ud_client,
            sns_client,
            ton_client,
            tree_client,
            spaceid_client,
        }
    }

    pub async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        let name_prefix = name.split('.').clone().last().unwrap_or_default();
        let provider = self.domains_mapping.get(name_prefix).expect("unable to get provider");

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
            },
            NameProvider::Tree => {
                if !TreeClient::chains().contains(&chain) {
                    return Err("not supported chain".to_string().into())
                }
                self.tree_client.resolve(name, chain).await
            },
            NameProvider::SpaceId => {
                if !SpaceIdClient::chains().contains(&chain) {
                    return Err("not supported chain".to_string().into())
                }
                self.spaceid_client.resolve(name, chain).await
            },
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

        for domain in TreeClient::domains() {
            result.insert(domain, NameProvider::Tree);
        }

        for domain in SpaceIdClient::domains() {
            result.insert(domain, NameProvider::SpaceId);
        }

        result
    }
}
