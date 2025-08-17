use std::error::Error;

#[cfg(feature = "reqwest")]
use gem_jsonrpc::JsonRpcClient;
use primitives::EVMChain;
use serde_json::json;

use crate::{
    registry::ContractRegistry,
    rpc::{
        ankr::model::{ankr_chain, TokenBalances, Transactions},
        EthereumClient, EthereumMapper,
    },
};

#[cfg(feature = "reqwest")]
#[derive(Clone)]
pub struct AnkrClient {
    pub chain: EVMChain,
    pub client: EthereumClient,
    rpc_client: JsonRpcClient,
}

#[cfg(feature = "reqwest")]
impl AnkrClient {
    pub fn new(client: EthereumClient, api_key: String) -> Self {
        let url = format!("https://rpc.ankr.com/multichain/{api_key}");
        let rpc_client = JsonRpcClient::new_reqwest(url);

        Self {
            chain: client.chain,
            client,
            rpc_client,
        }
    }

    pub async fn get_transactions_by_address(&self, address: &str, limit: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let transaction_ids = self.get_transactions_ids_by_address(address, limit).await?;
        if transaction_ids.is_empty() {
            return Ok(vec![]);
        }
        let contract_registry = ContractRegistry::default();
        Ok(self
            .client
            .get_transactions(&transaction_ids)
            .await?
            .into_iter()
            .filter_map(|(block, transaction, receipt, trace)| {
                EthereumMapper::map_transaction(
                    self.chain.to_chain(),
                    &transaction,
                    &receipt,
                    Some(&trace),
                    &block.timestamp,
                    Some(&contract_registry),
                )
            })
            .collect())
    }

    pub async fn get_transactions_ids_by_address(&self, address: &str, limit: i64) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .get_ankr_transactions_by_address(address, limit)
            .await?
            .transactions
            .into_iter()
            .map(|x| x.hash)
            .collect::<Vec<String>>())
    }

    /// Reference: https://www.ankr.com/docs/advanced-api/query-methods/#ankr_gettransactionsbyaddress
    pub async fn get_ankr_transactions_by_address(&self, address: &str, limit: i64) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        if let Some(chain) = ankr_chain(self.chain) {
            let params = serde_json::json!({
                "address": address,
                "blockchain": chain,
                "page_size": limit,
                "desc_order": true
            });
            Ok(self.rpc_client.call("ankr_getTransactionsByAddress", params).await?)
        } else {
            Ok(Transactions { transactions: vec![] })
        }
    }

    /// Reference: https://www.ankr.com/docs/advanced-api/token-methods/#ankr_getaccountbalance
    pub async fn get_token_balances(&self, address: &str) -> Result<TokenBalances, Box<dyn Error + Send + Sync>> {
        if let Some(chain) = ankr_chain(self.chain) {
            let params = json!([
                {
                    "walletAddress": address,
                    "blockchain": chain,
                    "onlyWhitelisted": true,
                }
            ]);

            Ok(self.rpc_client.call("ankr_getAccountBalance", params).await?)
        } else {
            Ok(TokenBalances { assets: vec![] })
        }
    }
}
