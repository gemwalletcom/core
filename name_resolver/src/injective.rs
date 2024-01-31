use crate::client::NameClient;
use async_trait::async_trait;
use primitives::{
    chain::Chain,
    name::{NameProvider, NameRecord},
};
use std::error::Error;

pub struct InjectiveNameClient {
    url: String,
}

impl InjectiveNameClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[async_trait]
impl NameClient for InjectiveNameClient {
    fn provider() -> NameProvider {
        NameProvider::Injective
    }

    // inj1hm8vs8sr2h9nk0x66vctfs528wrp6k3gtgg275
    // toBase64({
    //     resolver: {
    //       node: this.params.node,
    //     },
    async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        let hash = crate::ens_provider::namehash::namehash(name);

        println!("url: {:?}", self.url);
        println!("hash: {:?}", hash);

        Ok(NameRecord {
            name: name.to_string(),
            chain,
            address: "".to_string(),
            provider: Self::provider(),
        })
    }

    fn domains() -> Vec<&'static str> {
        vec!["inj"]
    }

    fn chains() -> Vec<Chain> {
        vec![Chain::Injective]
    }
}
