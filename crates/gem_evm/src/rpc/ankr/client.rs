use std::error::Error;

use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient as GenericJsonRpcClient;
use primitives::EVMChain;
use serde_json::json;

use crate::rpc::ankr::model::{ankr_chain, TokenBalances, Transactions};


#[derive(Clone)]
pub struct AnkrClient<C: Client + Clone> {
    pub chain: EVMChain,
    rpc_client: GenericJsonRpcClient<C>,
}

impl<C: Client + Clone> AnkrClient<C> {
    pub fn new(client: GenericJsonRpcClient<C>, chain: EVMChain) -> Self {
        Self {
            chain,
            rpc_client: client,
        }
    }
}

impl<C: Client + Clone> AnkrClient<C> {
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
