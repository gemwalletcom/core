use std::error::Error;

use crate::rpc::alchemy::{model::alchemy_rpc_url, Transactions};
use alloy_rpc_client::{ClientBuilder, RpcClient};
use primitives::EVMChain;
use url::Url;

use crate::rpc::alchemy::TokenBalances;

#[derive(Clone)]
pub struct AlchemyClient {
    pub chain: EVMChain,
    rpc_client: RpcClient,
}

impl AlchemyClient {
    const DISABLED_RPC_CHAINS: [EVMChain; 5] = [EVMChain::Mantle, EVMChain::Hyperliquid, EVMChain::OpBNB, EVMChain::Monad, EVMChain::Fantom];

    pub fn new(chain: EVMChain, api_key: String) -> Self {
        let rpc_client = ClientBuilder::default().http(Url::parse(&alchemy_rpc_url(chain, &api_key)).expect("Invalid Alchemy API URL"));

        Self { chain, rpc_client }
    }

    // https://www.alchemy.com/docs/data/token-api/token-api-endpoints/alchemy-get-token-balances
    pub async fn get_token_balances(&self, address: &str) -> Result<TokenBalances, Box<dyn Error + Send + Sync>> {
        if Self::DISABLED_RPC_CHAINS.contains(&self.chain) {
            return Ok(TokenBalances {
                address: Some(address.to_string()),
                token_balances: vec![],
            });
        }
        Ok(self.rpc_client.request("alchemy_getTokenBalances", (address,)).await?)
    }

    // https://www.alchemy.com/docs/data/transfers-api/transfers-endpoints/alchemy-get-asset-transfers
    pub async fn get_asset_transfers(&self, address: &str) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        if Self::DISABLED_RPC_CHAINS.contains(&self.chain) {
            return Ok(Transactions { transactions: vec![] });
        }
        let params = serde_json::json!([{
            "fromBlock": "0x0",
            //"fromAddress": address,
            "toAddress": address,
            "excludeZeroValue": true,
            "category": [
                "external",
                "internal",
                "erc20",
            ],
            "order": "desc"
        }]);

        Ok(self.rpc_client.request("alchemy_getAssetTransfers", params).await?)
    }
}
