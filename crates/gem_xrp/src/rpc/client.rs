use std::error::Error;

use reqwest_middleware::ClientWithMiddleware;
use serde_json::json;

use crate::rpc::model::AccountLedger;

use super::model::{AccountObjects, Ledger, LedgerCurrent, LedgerData, LedgerResult};

pub struct XRPClient {
    url: String,
    client: ClientWithMiddleware,
}

impl XRPClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub async fn get_ledger_current(&self) -> Result<LedgerCurrent, Box<dyn Error + Send + Sync>> {
        let params = json!(
            {
                "method": "ledger_current",
                "params": [{}]
            }
        );
        Ok(self
            .client
            .post(self.url.clone())
            .json(&params)
            .send()
            .await?
            .json::<LedgerResult<LedgerCurrent>>()
            .await?
            .result)
    }

    pub async fn get_block_transactions(&self, block_number: i64) -> Result<Ledger, Box<dyn Error + Send + Sync>> {
        let params = json!(
            {
                "method": "ledger",
                "params": [
                    {
                        "ledger_index": block_number,
                        "transactions": true,
                        "expand": true
                    }
                ]
            }
        );
        Ok(self
            .client
            .post(self.url.clone())
            .json(&params)
            .send()
            .await?
            .json::<LedgerResult<LedgerData>>()
            .await?
            .result
            .ledger)
    }

    pub async fn get_account_transactions(&self, address: String, limit: i64) -> Result<AccountLedger, Box<dyn Error + Send + Sync>> {
        let params = json!(
            {
                "method": "account_tx",
                "params": [
                    {
                        "account": address,
                        "limit": limit,
                        "api_version": 2
                    }
                ]
            }
        );
        Ok(self
            .client
            .post(self.url.clone())
            .json(&params)
            .send()
            .await?
            .json::<LedgerResult<AccountLedger>>()
            .await?
            .result)
    }

    pub async fn get_account_objects(&self, token_id: String) -> Result<AccountObjects, Box<dyn Error + Send + Sync>> {
        let params = json!({ "method": "account_objects", "params": [ { "ledger_index": "validated", "type": "state", "account": token_id } ] });
        Ok(self
            .client
            .post(self.url.clone())
            .json(&params)
            .send()
            .await?
            .json::<LedgerResult<AccountObjects>>()
            .await?
            .result)
    }
}
