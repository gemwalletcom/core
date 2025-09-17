use serde_json::json;
use std::error::Error;

use crate::models::rpc::*;

use chain_traits::{ChainAddressStatus, ChainPerpetual, ChainProvider, ChainStaking, ChainTraits};
use gem_client::Client;
use primitives::Chain;

#[derive(Debug)]
pub struct XRPClient<C: Client> {
    client: C,
    pub chain: Chain,
}

impl<C: Client> XRPClient<C> {
    pub fn new(client: C) -> Self {
        Self { client, chain: Chain::Xrp }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_account_info(&self, address: &str) -> Result<AccountInfo, Box<dyn Error + Send + Sync>> {
        let result = self.get_account_info_full(address).await?;
        if let Some(account_data) = result.account_data {
            Ok(account_data)
        } else {
            Err("Account not found".into())
        }
    }

    pub async fn get_account_info_full(&self, address: &str) -> Result<AccountInfoResult, Box<dyn Error + Send + Sync>> {
        let params = json!({
            "method": "account_info",
            "params": [
                {
                    "account": address,
                    "ledger_index": "current"
                }
            ]
        });

        let result: LedgerResult<AccountInfoResult> = self.client.post("", &params, None).await?;
        Ok(result.result)
    }

    pub async fn get_ledger_current(&self) -> Result<LedgerCurrent, Box<dyn Error + Send + Sync>> {
        let params = json!({
            "method": "ledger_current",
            "params": [{}]
        });

        let result: LedgerResult<LedgerCurrent> = self.client.post("", &params, None).await?;
        Ok(result.result)
    }

    pub async fn get_last_ledger_sequence(&self) -> Result<u32, Box<dyn Error + Send + Sync>> {
        let current = self.get_ledger_current().await?;
        Ok((current.ledger_current_index + 20) as u32)
    }

    pub async fn get_fees(&self) -> Result<FeesResult, Box<dyn Error + Send + Sync>> {
        let params = json!({
            "method": "fee",
            "params": [{}]
        });

        let result: LedgerResult<FeesResult> = self.client.post("", &params, None).await?;
        Ok(result.result)
    }

    pub async fn broadcast_transaction(&self, data: &str) -> Result<TransactionBroadcast, Box<dyn Error + Send + Sync>> {
        let params = json!({
            "method": "submit",
            "params": [
                {
                    "tx_blob": data,
                    "fail_hard": true
                }
            ]
        });

        let result: LedgerResult<TransactionBroadcast> = self.client.post("", &params, None).await?;

        Ok(result.result)
    }

    pub async fn get_transaction_status(&self, transaction_id: &str) -> Result<TransactionStatus, Box<dyn Error + Send + Sync>> {
        let params = json!({
            "method": "tx",
            "params": [
                {
                    "transaction": transaction_id
                }
            ]
        });

        let result: LedgerResult<TransactionStatus> = self.client.post("", &params, None).await?;
        Ok(result.result)
    }

    pub async fn get_account_objects(&self, address: &str) -> Result<AccountObjects, Box<dyn Error + Send + Sync>> {
        let params = json!({
            "method": "account_objects",
            "params": [
                {
                    "account": address,
                    "type": "state",
                    "ledger_index": "validated"
                }
            ]
        });

        let result: LedgerResult<AccountObjects> = self.client.post("", &params, None).await?;
        Ok(result.result)
    }

    pub async fn get_block_transactions(&self, block_number: i64) -> Result<Ledger, Box<dyn Error + Send + Sync>> {
        let params = json!({
            "method": "ledger",
            "params": [
                {
                    "ledger_index": block_number,
                    "transactions": true,
                    "expand": true
                }
            ]
        });

        let result: LedgerResult<LedgerData> = self.client.post("", &params, None).await?;
        Ok(result.result.ledger)
    }

    pub async fn get_account_transactions(&self, address: String, limit: usize) -> Result<AccountLedger, Box<dyn Error + Send + Sync>> {
        let params = json!({
            "method": "account_tx",
            "params": [
                {
                    "account": address,
                    "limit": limit,
                    "ledger_index_max": -1,
                    "ledger_index_min": -1
                }
            ]
        });

        let result: LedgerResult<AccountLedger> = self.client.post("", &params, None).await?;
        Ok(result.result)
    }
}

impl<C: Client> ChainStaking for XRPClient<C> {}

impl<C: Client> ChainPerpetual for XRPClient<C> {}

impl<C: Client> ChainAddressStatus for XRPClient<C> {}

impl<C: Client> chain_traits::ChainAccount for XRPClient<C> {}

impl<C: Client> ChainTraits for XRPClient<C> {}

impl<C: Client> ChainProvider for XRPClient<C> {
    fn get_chain(&self) -> Chain {
        self.chain
    }
}
