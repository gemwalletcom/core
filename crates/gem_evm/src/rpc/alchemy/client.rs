use std::error::Error;

use crate::rpc::{alchemy::model::alchemy_rpc_url, EthereumClient, EthereumMapper};
use gem_jsonrpc::JsonRpcClient;
use primitives::EVMChain;
use serde_json::json;

use crate::rpc::alchemy::TokenBalances;

#[derive(Clone)]
pub struct AlchemyClient {
    pub chain: EVMChain,
    api_key: String,
    ethereum_client: EthereumClient,
    rpc_client: JsonRpcClient,
}

impl AlchemyClient {
    const DISABLED_RPC_CHAINS: [EVMChain; 5] = [EVMChain::Mantle, EVMChain::Hyperliquid, EVMChain::OpBNB, EVMChain::Monad, EVMChain::Fantom];

    pub fn new(ethereum_client: EthereumClient, api_key: String) -> Self {
        let chain = ethereum_client.chain;
        let rpc_client = JsonRpcClient::new(alchemy_rpc_url(chain, &api_key)).expect("Invalid Alchemy API URL");

        Self {
            chain,
            api_key,
            ethereum_client,
            rpc_client,
        }
    }

    pub async fn get_transactions_by_address(&self, address: &str) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions_ids = self.get_transactions_ids_by_address(address).await?;
        if transactions_ids.is_empty() {
            return Ok(vec![]);
        }
        Ok(self
            .ethereum_client
            .get_transactions(transactions_ids.clone())
            .await?
            .into_iter()
            .filter_map(|(block, transaction, receipt)| EthereumMapper::map_transaction(self.chain.to_chain(), &transaction, &receipt, &block.timestamp))
            .collect())
    }

    // https://www.alchemy.com/docs/data/token-api/token-api-endpoints/alchemy-get-token-balances
    pub async fn get_token_balances(&self, address: &str) -> Result<TokenBalances, Box<dyn Error + Send + Sync>> {
        if Self::DISABLED_RPC_CHAINS.contains(&self.chain) {
            return Ok(TokenBalances {
                address: Some(address.to_string()),
                token_balances: vec![],
            });
        }
        Ok(self.rpc_client.call("alchemy_getTokenBalances", json!([address])).await?)
    }
    // https://www.alchemy.com/docs/data/portfolio-apis/portfolio-api-endpoints/portfolio-api-endpoints/get-transaction-history-by-address
    //TODO: implement
    pub async fn get_transactions_ids_by_address(&self, _address: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let _url = format!("https://api.g.alchemy.com/data/v1/{}/transactions/history/by-address", &self.api_key);
        //let client = ClientBuilder::default().http(Url::parse(&url).expect("Invalid Alchemy API URL"));

        Ok(vec![])
    }
}
