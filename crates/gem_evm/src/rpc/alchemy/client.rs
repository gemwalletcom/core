use alloy_rpc_client::{ClientBuilder, RpcClient};
use anyhow::Result;
use primitives::EVMChain;
use url::Url;

use crate::rpc::alchemy::{model::alchemy_url, TokenBalances};

pub struct AlchemyClient {
    client: RpcClient,
    pub chain: EVMChain,
}

impl AlchemyClient {
    pub fn new(chain: EVMChain, api_key: &str) -> Self {
        let url = alchemy_url(chain, api_key);
        let parsed_url = Url::parse(&url).expect("Invalid Alchemy API URL");
        let client = ClientBuilder::default().http(parsed_url);
        Self { client, chain }
    }

    pub async fn get_token_balances(&self, address: &str) -> Result<TokenBalances> {
        Ok(self.client.request("alchemy_getTokenBalances", (address,)).await?)
    }
}
