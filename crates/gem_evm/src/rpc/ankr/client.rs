use std::error::Error;

use alloy_rpc_client::{ClientBuilder, RpcClient};
use primitives::EVMChain;
use serde_json::json;
use url::Url;

use crate::rpc::ankr::model::{ankr_chain, TokenBalances, Transactions};

#[derive(Clone)]
pub struct AnkrClient {
    pub chain: EVMChain,
    rpc_client: RpcClient,
}

impl AnkrClient {
    pub fn new(chain: EVMChain, api_key: String) -> Self {
        let url = format!("https://rpc.ankr.com/multichain/{}", api_key);
        let rpc_client = ClientBuilder::default().http(Url::parse(&url).expect("Invalid Ankr API URL"));

        Self { chain, rpc_client }
    }

    /// Reference: https://www.ankr.com/docs/advanced-api/query-methods/#ankr_gettransactionsbyaddress
    pub async fn get_transactions_by_address(&self, address: &str) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        if let Some(chain) = ankr_chain(self.chain) {
            let params = serde_json::json!({
                "address": address,
                "blockchain": chain,
                "page_size": 50,
                "desc_order": true
            });
            Ok(self.rpc_client.request("ankr_getTransactionsByAddress", params).await?)
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

            Ok(self.rpc_client.request("ankr_getAccountBalance", params).await?)
        } else {
            Ok(TokenBalances { assets: vec![] })
        }
    }
}
