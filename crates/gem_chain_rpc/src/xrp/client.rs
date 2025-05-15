use std::error::Error;


use reqwest_middleware::ClientWithMiddleware;
use serde_json::json;

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
        let response = self
            .client
            .post(self.url.clone())
            .json(&params)
            .send()
            .await?
            .json::<LedgerResult<LedgerCurrent>>()
            .await?;

        Ok(response.result)
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
        let response = self
            .client
            .post(self.url.clone())
            .json(&params)
            .send()
            .await?
            .json::<LedgerResult<LedgerData>>()
            .await?;

        Ok(response.result.ledger)
    }
    
    pub async fn get_account_objects(&self, token_id: String) -> Result<AccountObjects, Box<dyn Error + Send + Sync>> {
        let params = json!({ "method": "account_objects", "params": [ { "ledger_index": "validated", "state": "type", "account": token_id } ] });

        let response = self
            .client
            .post(self.url.clone())
            .json(&params)
            .send()
            .await?
            .json::<LedgerResult<AccountObjects>>()
            .await?;
            
        Ok(response.result)
    }
}


