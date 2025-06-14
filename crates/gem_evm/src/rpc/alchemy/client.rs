use alloy_rpc_client::{ClientBuilder, RpcClient};
use anyhow::Result;
use primitives::EVMChain;
use url::Url;

use crate::rpc::alchemy::{model::alchemy_url, AssetTransfers, TokenBalances};

#[derive(Clone)]
pub struct AlchemyClient {
    client: RpcClient,
    pub chain: EVMChain,
}

impl AlchemyClient {
    const DISABLED_CHAINS: [EVMChain; 5] = [EVMChain::Mantle, EVMChain::Hyperliquid, EVMChain::OpBNB, EVMChain::Monad, EVMChain::Fantom];

    pub fn new(chain: EVMChain, api_key: &str) -> Self {
        let url = alchemy_url(chain, api_key);
        let parsed_url = Url::parse(&url).expect("Invalid Alchemy API URL");
        let client = ClientBuilder::default().http(parsed_url);
        Self { client, chain }
    }

    // https://www.alchemy.com/docs/data/token-api/token-api-endpoints/alchemy-get-token-balances
    pub async fn get_token_balances(&self, address: &str) -> Result<TokenBalances> {
        if Self::DISABLED_CHAINS.contains(&self.chain) {
            return Ok(TokenBalances {
                address: Some(address.to_string()),
                token_balances: vec![],
            });
        }
        Ok(self.client.request("alchemy_getTokenBalances", (address,)).await?)
    }

    // https://www.alchemy.com/docs/data/transfers-api/transfers-endpoints/alchemy-get-asset-transfers
    pub async fn get_asset_transfers(&self, address: &str) -> Result<AssetTransfers> {
        if Self::DISABLED_CHAINS.contains(&self.chain) {
            return Ok(AssetTransfers { transfers: vec![] });
        }
        let params = serde_json::json!([{
            "fromBlock": "0x0",
            //"fromAddress": address,
            "toAddress": address,
            "excludeZeroValue": true,
            "withMetadata": true,
            "category": [
                "external",
                "internal",
                "erc20",
            ],
            "order": "desc"
        }]);

        Ok(self.client.request("alchemy_getAssetTransfers", params).await?)
    }
}
