use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use primitives::chain::Chain;
use primitives::name::{NameProvider, NameRecord};

use crate::error::NameError;

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
        let provider = self.matched_provider(name, chain)?;
        let address = provider.resolve(name, chain).await?;

        Ok(NameRecord {
            provider: provider.provider().as_ref().to_string(),
            address,
            name: name.to_string(),
            chain,
        })
    }

    fn matched_provider(&self, name: &str, chain: Chain) -> Result<&(dyn NameClient + Send + Sync), Box<dyn Error + Send + Sync>> {
        self.providers
            .iter()
            .enumerate()
            .filter(|(_, provider)| provider.chains().contains(&chain))
            .filter_map(|(index, provider)| {
                provider
                    .domains()
                    .iter()
                    .filter_map(|domain| domain_match_len(name, domain))
                    .max()
                    .map(|match_len| (match_len, index, provider.as_ref()))
            })
            .max_by(|left, right| left.0.cmp(&right.0).then(right.1.cmp(&left.1)))
            .map(|(_, _, provider)| provider)
            .ok_or_else(|| NameError::new(format!("No provider found for name: {name}")).into())
    }
}

fn domain_match_len(name: &str, domain: &str) -> Option<usize> {
    if domain == "*" {
        return Some(0);
    }

    let name = name.to_ascii_lowercase();
    let domain = domain.to_ascii_lowercase();

    if name == domain || name.ends_with(&format!(".{domain}")) {
        Some(domain.len())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::testkit::TestProvider;
    use primitives::name::NameProvider;

    use super::Client;
    use primitives::chain::Chain;

    #[tokio::test]
    async fn test_resolve_prefers_longer_domain_match() {
        let client = Client::new(vec![
            TestProvider::new(NameProvider::Ens, vec!["*"], vec![Chain::Base], Ok("0x0000000000000000000000000000000000000001")),
            TestProvider::new(
                NameProvider::Basenames,
                vec!["base.eth"],
                vec![Chain::Base],
                Ok("0x0000000000000000000000000000000000000002"),
            ),
        ]);

        let record = client.resolve("alice.base.eth", Chain::Base).await.unwrap();

        assert_eq!(record.provider, NameProvider::Basenames.as_ref());
        assert_eq!(record.address, "0x0000000000000000000000000000000000000002");
    }

    #[tokio::test]
    async fn test_resolve_returns_more_specific_provider_error() {
        let client = Client::new(vec![
            TestProvider::new(NameProvider::Ens, vec!["*"], vec![Chain::Base], Ok("0x0000000000000000000000000000000000000003")),
            TestProvider::new(NameProvider::Basenames, vec!["base.eth"], vec![Chain::Base], Err("failed")),
        ]);

        let error = client.resolve("alice.base.eth", Chain::Base).await.err().unwrap();

        assert_eq!(error.to_string(), "failed");
    }
}
