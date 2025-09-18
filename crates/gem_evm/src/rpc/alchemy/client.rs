use std::error::Error;

use crate::rpc::alchemy::{
    model::{evm_chain_to_network, Data},
    TokenBalances, Transactions,
};
use gem_client::{Client, ContentType, CONTENT_TYPE};
use primitives::EVMChain;
use serde_json::json;

#[derive(Clone)]
pub struct AlchemyClient<C: Client + Clone> {
    pub chain: EVMChain,
    client: C,
}

pub fn alchemy_url(api_key: &str) -> String {
    format!("https://api.g.alchemy.com/data/v1/{api_key}")
}

impl<C: Client + Clone> AlchemyClient<C> {
    const DISABLED_RPC_CHAINS: [EVMChain; 5] = [EVMChain::Mantle, EVMChain::Hyperliquid, EVMChain::OpBNB, EVMChain::Monad, EVMChain::Fantom];
    const ENABLED_TRANSACTION_CHAINS: [EVMChain; 2] = [EVMChain::Ethereum, EVMChain::Base];

    fn common_headers() -> std::collections::HashMap<String, String> {
        let mut headers = std::collections::HashMap::new();
        headers.insert(CONTENT_TYPE.to_string(), ContentType::ApplicationJson.as_str().to_string());
        headers
    }

    pub fn new(client: C, chain: EVMChain) -> Self {
        Self { chain, client }
    }

    pub async fn get_transactions_ids_by_address(&self, address: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let transactions = self.get_transactions_address(address).await?.data.transactions;
        Ok(transactions.iter().map(|x| x.hash.clone()).collect())
    }

    // https://www.alchemy.com/docs/data/token-api/token-api-endpoints/alchemy-get-token-balances
    pub async fn get_token_balances(&self, address: &str) -> Result<Data<TokenBalances>, Box<dyn Error + Send + Sync>> {
        if Self::DISABLED_RPC_CHAINS.contains(&self.chain) {
            return Ok(Data {
                data: TokenBalances { tokens: vec![] },
            });
        }
        let chain = evm_chain_to_network(self.chain);
        let payload = json!({
            "addresses": [
                {
                    "address": address,
                    "networks": [chain]
                }
            ],
            "includeNativeTokens": false,
        });
        Ok(self
            .client
            .post("/assets/tokens/balances/by-address", &payload, Some(Self::common_headers()))
            .await?)
    }
    // https://www.alchemy.com/docs/data/portfolio-apis/portfolio-api-endpoints/portfolio-api-endpoints/get-transaction-history-by-address
    //TODO:
    pub async fn get_transactions_address(&self, address: &str) -> Result<Data<Transactions>, Box<dyn Error + Send + Sync>> {
        if Self::DISABLED_RPC_CHAINS.contains(&self.chain) || !Self::ENABLED_TRANSACTION_CHAINS.contains(&self.chain) {
            return Ok(Data {
                data: Transactions { transactions: vec![] },
            });
        }
        let chain = evm_chain_to_network(self.chain);
        let payload = json!({
            "addresses": [
                {
                    "address": address,
                    "networks": [chain]
                }
            ],
            "limit": 25,
        });
        Ok(self
            .client
            .post("/transactions/history/by-address", &payload, Some(Self::common_headers()))
            .await?)
    }
}
