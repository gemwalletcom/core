use super::contract::Contract;
use jsonrpsee::core::Error;
use jsonrpsee::http_client::HttpClientBuilder;
use primitives::Chain;

static REGISTRY: &str = "0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e";
pub struct Provider {
    contract: Contract,
}

impl Provider {
    pub fn new(url: String) -> Self {
        let client: jsonrpsee::http_client::HttpClient =
            HttpClientBuilder::default().build(url).unwrap();
        return Provider {
            contract: Contract {
                client: client,
                registry: REGISTRY.to_string(),
            },
        };
    }

    pub async fn resolve_name(&self, name: &str, _chain: Chain) -> Result<String, Error> {
        let resolver = self.contract.resolver(name).await?;
        if resolver.is_empty() {
            return Err(Error::Custom(String::from("no resolver set")));
        }
        // TODO: support other chain lookup
        // TODO: support recursive parent lookup
        // TODO: support off chain lookup CCIP-Read
        let addr = self.contract.legacy_addr(&resolver, name).await?;
        Ok(addr)
    }

    pub async fn get_address(&self, _resolver: &str, _chain: Chain) -> Result<String, Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::Provider;
    use primitives::Chain;
    use tokio_test::block_on;

    #[test]
    fn test_resolver() {
        block_on(async {
            let provider = Provider::new(String::from("https://eth.llamarpc.com"));
            let addres = provider.resolve_name("vitalik.eth", Chain::Ethereum).await;
            assert_eq!(
                addres.unwrap(),
                "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_lowercase()
            )
        });
    }
}
