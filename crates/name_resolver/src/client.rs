use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use primitives::chain::Chain;
use primitives::name::{NameProvider, NameRecord};

#[async_trait]
pub trait NameClient {
    async fn resolve(&self, name: &str, chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>>;
    fn provider(&self) -> NameProvider;
    fn domains(&self) -> Vec<&'static str>;
    fn chains(&self) -> Vec<Chain>;
}

#[async_trait]
impl<T: Send + Sync> NameClient for Arc<T>
where
    T: NameClient + ?Sized,
{
    async fn resolve(&self, name: &str, chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        (**self).resolve(name, chain).await
    }

    fn provider(&self) -> NameProvider {
        (**self).provider()
    }

    fn domains(&self) -> Vec<&'static str> {
        (**self).domains()
    }
    fn chains(&self) -> Vec<Chain> {
        (**self).chains()
    }
}

pub struct Client {
    providers: Vec<Box<dyn NameClient + Send + Sync>>,
}

impl Client {
    pub fn new(providers: Vec<Box<dyn NameClient + Send + Sync>>) -> Self {
        Self { providers }
    }

    pub async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error + Send + Sync>> {
        let name_prefix = name.split('.').clone().next_back().unwrap_or_default();
        for provider in self.providers.iter() {
            if provider.chains().contains(&chain) && provider.domains().contains(&name_prefix) {
                match provider.resolve(name, chain).await {
                    Ok(address) => {
                        let record = NameRecord {
                            provider: provider.provider().as_ref().to_string(),
                            address,
                            name: name.to_string(),
                            chain,
                        };
                        return Ok(record);
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        Err(format!("No provider found for name: {name}").into())
    }
}
