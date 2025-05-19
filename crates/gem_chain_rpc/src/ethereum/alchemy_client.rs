use serde_serializers::deserialize_biguint_from_hex_str;
use std::error::Error;

use async_trait::async_trait;
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use num_bigint::BigUint;
use primitives::{chain::Chain, AssetBalance, AssetId};
use serde::Deserialize;

use crate::ChainAssetsProvider;

pub struct AlchemyClient {
    client: HttpClient,
    chain: Chain,
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

        let client = HttpClientBuilder::default()
            .max_response_size(256 * 1024 * 1024) // 256MB
            .build(url)
            .unwrap();

        Self { client, chain }
    }

    pub async fn get_token_balances(&self, address: &str) -> Result<TokenBalances, Box<dyn Error + Send + Sync>> {
        Ok(self.client.request("alchemy_getTokenBalances", rpc_params![address]).await?)
    }
}

#[async_trait]
impl ChainAssetsProvider for AlchemyClient {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let response = self.get_token_balances(&address).await?;
        let balances = response
            .token_balances
            .into_iter()
            .map(|x| AssetBalance {
                asset_id: AssetId::from_token(self.chain, &x.contract_address),
                balance: x.token_balance.to_string(),
            })
            .collect();
        Ok(balances)
    }
}
