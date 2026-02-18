use std::error::Error;

use async_trait::async_trait;
use primitives::chain::Chain;
use primitives::name::NameProvider;

use crate::client::NameClient;
use crate::error::NameError;

pub struct TestProvider {
    provider: NameProvider,
    domains: Vec<&'static str>,
    chains: Vec<Chain>,
    response: Result<String, &'static str>,
}

impl TestProvider {
    pub fn new(provider: NameProvider, domains: Vec<&'static str>, chains: Vec<Chain>, response: Result<&'static str, &'static str>) -> Box<dyn NameClient + Send + Sync> {
        let response = match response {
            Ok(address) => Ok(address.to_string()),
            Err(error) => Err(error),
        };

        Box::new(Self {
            provider,
            domains,
            chains,
            response,
        })
    }
}

#[async_trait]
impl NameClient for TestProvider {
    async fn resolve(&self, _name: &str, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        match &self.response {
            Ok(address) => Ok(address.clone()),
            Err(error) => Err(Box::new(NameError::new(error.to_string()))),
        }
    }

    fn provider(&self) -> NameProvider {
        self.provider.clone()
    }

    fn domains(&self) -> Vec<&'static str> {
        self.domains.clone()
    }

    fn chains(&self) -> Vec<Chain> {
        self.chains.clone()
    }
}
