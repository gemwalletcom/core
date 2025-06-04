use alloy_rpc_client::RpcClient;
use alloy_transport_http::{reqwest::Client as ReqwestClient, Http};
use anyhow::Result;
use async_trait::async_trait;
use num_bigint::BigUint;
use serde::Deserialize;
use std::error::Error;
use url::Url;

use gem_chain_rpc::ChainAssetsProvider;
use primitives::{chain::Chain, AssetBalance, AssetId};
use serde_serializers::deserialize_biguint_from_hex_str;

pub struct AlchemyClient {
    client: RpcClient,
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

        let parsed_url = Url::parse(&url).expect("Invalid Alchemy API URL");
        let reqwest_client = ReqwestClient::new();
        let http_transport = Http::with_client(reqwest_client, parsed_url);
        let client = RpcClient::new(http_transport, true);

        Self { client, chain }
    }

    pub async fn get_token_balances(&self, address: &str) -> Result<TokenBalances> {
        let response = self.client.request("alchemy_getTokenBalances", (address,)).await?;
        Ok(response)
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
