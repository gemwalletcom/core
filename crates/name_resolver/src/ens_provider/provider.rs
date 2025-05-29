use super::contract::Contract;
use anyhow::Result;
use primitives::Chain;
use std::error::Error;

static REGISTRY: &str = "0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e";
pub struct Provider {
    contract: Contract,
}

impl Provider {
    pub fn new(url: String) -> Result<Self> {
        let contract = Contract::new(&url, REGISTRY)?;
        Ok(Provider { contract })
    }

    pub async fn resolve_name(&self, name: &str, _chain: Chain) -> Result<String> {
        let resolver_address = self.contract.resolver(name).await?;
        // TODO: support other chain lookup
        // TODO: support recursive parent lookup
        // TODO: support off chain lookup CCIP-Read
        let addr = self.contract.legacy_addr(&resolver_address.to_string(), name).await?;
        Ok(addr.to_checksum(None))
    }

    pub async fn get_address(&self, _resolver: &str, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        todo!()
    }
}
