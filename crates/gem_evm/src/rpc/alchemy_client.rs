use alloy_rpc_client::{ClientBuilder, RpcClient};
use anyhow::Result;
use num_bigint::BigUint;
use serde::Deserialize;
use url::Url;

use primitives::chain::Chain;
use serde_serializers::deserialize_biguint_from_hex_str;

pub struct AlchemyClient {
    client: RpcClient,
    pub chain: Chain,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub contract_address: String,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub token_balance: BigUint,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalances {
    pub token_balances: Vec<TokenBalance>,
}

impl AlchemyClient {
    pub fn new(chain: Chain, api_key: &str) -> Self {
        let url = format!(
            "https://{}-mainnet.g.alchemy.com/v2/{}",
            match chain {
                Chain::Ethereum => "eth",
                _ => chain.as_ref(),
            },
            api_key,
        );

        let parsed_url = Url::parse(&url).expect("Invalid Alchemy API URL");
        let client = ClientBuilder::default().http(parsed_url);
        Self { client, chain }
    }

    pub async fn get_token_balances(&self, address: &str) -> Result<TokenBalances> {
        let response = self.client.request("alchemy_getTokenBalances", (address,)).await?;
        Ok(response)
    }
}
