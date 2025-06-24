use std::error::Error;

use crate::rpc::{
    alchemy::{
        model::{evm_chain_to_network, Data},
        Transactions,
    },
    EthereumClient, EthereumMapper,
};
use primitives::EVMChain;
use reqwest_middleware::ClientWithMiddleware;
use serde_json::json;

use crate::rpc::alchemy::TokenBalances;

#[derive(Clone)]
pub struct AlchemyClient {
    pub chain: EVMChain,
    url: String,
    client: ClientWithMiddleware,
    ethereum_client: EthereumClient,
}

impl AlchemyClient {
    const DISABLED_RPC_CHAINS: [EVMChain; 5] = [EVMChain::Mantle, EVMChain::Hyperliquid, EVMChain::OpBNB, EVMChain::Monad, EVMChain::Fantom];
    const ENABLED_TRANSACTION_CHAINS: [EVMChain; 2] = [EVMChain::Ethereum, EVMChain::Base];

    pub fn new(ethereum_client: EthereumClient, client: ClientWithMiddleware, api_key: String) -> Self {
        let chain = ethereum_client.chain;
        let url = format!("https://api.g.alchemy.com/data/v1/{}", api_key);

        Self {
            chain,
            url,
            client,
            ethereum_client,
        }
    }

    pub async fn get_transactions_by_address(&self, address: &str) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.get_transactions_address(address).await?.data.transactions;
        if transactions.is_empty() {
            return Ok(vec![]);
        }
        let transactions_ids = transactions.iter().map(|x| x.hash.clone()).collect::<Vec<String>>();
        Ok(self
            .ethereum_client
            .get_transactions(transactions_ids.clone())
            .await?
            .into_iter()
            .filter_map(|(block, transaction, receipt)| EthereumMapper::map_transaction(self.chain.to_chain(), &transaction, &receipt, &block.timestamp))
            .collect())
    }

    // https://www.alchemy.com/docs/data/token-api/token-api-endpoints/alchemy-get-token-balances
    pub async fn get_token_balances(&self, address: &str) -> Result<Data<TokenBalances>, Box<dyn Error + Send + Sync>> {
        if Self::DISABLED_RPC_CHAINS.contains(&self.chain) {
            return Ok(Data {
                data: TokenBalances { tokens: vec![] },
            });
        }
        let chain = evm_chain_to_network(self.chain);
        let url = format!("{}/assets/tokens/balances/by-address", &self.url);
        let payload = json!({
            "addresses": [
                {
                    "address": address,
                    "networks": [chain]
                }
            ],
            "includeNativeTokens": false,
        });
        Ok(self.client.post(url).json(&payload).send().await?.json().await?)
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
        let url = format!("{}/transactions/history/by-address", &self.url);
        let payload = json!({
            "addresses": [
                {
                    "address": address,
                    "networks": [chain]
                }
            ],
            "limit": 25,
        });
        Ok(self.client.post(url).json(&payload).send().await?.json().await?)
    }
}
