use super::contract::Contract;
use jsonrpsee::core::ClientError;
use jsonrpsee::http_client::HttpClientBuilder;
use primitives::Chain;

static REGISTRY: &str = "0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e";
pub struct Provider {
    contract: Contract,
}

impl Provider {
    pub fn new(url: String) -> Self {
        let client: jsonrpsee::http_client::HttpClient = HttpClientBuilder::default().build(url).unwrap();
        Provider {
            contract: Contract {
                client,
                registry: REGISTRY.to_string(),
            },
        }
    }

    pub async fn resolve_name(&self, name: &str, _chain: Chain) -> Result<String, ClientError> {
        let resolver = self.contract.resolver(name).await?;
        if resolver.is_empty() {
            return Err(ClientError::Custom(String::from("no resolver set")));
        }
        // TODO: support other chain lookup
        // TODO: support recursive parent lookup
        // TODO: support off chain lookup CCIP-Read
        let addr = self.contract.legacy_addr(&resolver, name).await?;
        Ok(addr)
    }

    pub async fn get_address(&self, _resolver: &str, _chain: Chain) -> Result<String, ClientError> {
        todo!()
    }
}
