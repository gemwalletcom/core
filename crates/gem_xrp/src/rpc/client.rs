use serde_json::json;
use std::error::Error;

use crate::models::rpc::{
    AccountInfo, AccountInfoResult, AccountLedger, AccountObjects, FeesResult, Ledger, LedgerCurrent, LedgerData, TransactionBroadcast, TransactionStatus,
};

use chain_traits::{ChainAddressStatus, ChainPerpetual, ChainProvider, ChainStaking, ChainTraits};
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient as GenericJsonRpcClient;
use primitives::Chain;

#[derive(Clone, Debug)]
pub struct XRPClient<C: Client + Clone> {
    client: GenericJsonRpcClient<C>,
    pub chain: Chain,
}

impl<C: Client + Clone> XRPClient<C> {
    pub fn new(client: GenericJsonRpcClient<C>) -> Self {
        Self { client, chain: Chain::Xrp }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub fn get_client(&self) -> &GenericJsonRpcClient<C> {
        &self.client
    }

    pub async fn get_account_info(&self, address: &str) -> Result<Option<AccountInfo>, Box<dyn Error + Send + Sync>> {
        let result = self.get_account_info_full(address).await?;
        Ok(result.account_data)
    }

    pub async fn get_account_info_full(&self, address: &str) -> Result<AccountInfoResult, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "account": address,
                "ledger_index": "current"
            }
        ]);

        Ok(self.client.call("account_info", params).await?)
    }

    pub async fn get_ledger_current(&self) -> Result<LedgerCurrent, Box<dyn Error + Send + Sync>> {
        let params = json!([{}]);
        Ok(self.client.call("ledger_current", params).await?)
    }

    pub async fn get_last_ledger_sequence(&self) -> Result<u32, Box<dyn Error + Send + Sync>> {
        let current = self.get_ledger_current().await?;
        Ok((current.ledger_current_index + 20) as u32)
    }

    pub async fn get_fees(&self) -> Result<FeesResult, Box<dyn Error + Send + Sync>> {
        let params = json!([{}]);
        Ok(self.client.call("fee", params).await?)
    }

    pub async fn broadcast_transaction(&self, data: &str) -> Result<TransactionBroadcast, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "tx_blob": data,
                "fail_hard": true
            }
        ]);

        Ok(self.client.call("submit", params).await?)
    }

    pub async fn get_transaction_status(&self, transaction_id: &str) -> Result<TransactionStatus, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "transaction": transaction_id
            }
        ]);

        Ok(self.client.call("tx", params).await?)
    }

    pub async fn get_account_objects(&self, address: &str) -> Result<AccountObjects, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "account": address,
                "type": "state",
                "ledger_index": "validated"
            }
        ]);

        Ok(self.client.call("account_objects", params).await?)
    }

    pub async fn get_block_transactions(&self, block_number: i64) -> Result<Ledger, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "ledger_index": block_number,
                "transactions": true,
                "expand": true
            }
        ]);

        let result: LedgerData = self.client.call("ledger", params).await?;
        Ok(result.ledger)
    }

    pub async fn get_account_transactions(&self, address: String, limit: usize) -> Result<AccountLedger, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "account": address,
                "limit": limit,
                "ledger_index_max": -1,
                "ledger_index_min": -1
            }
        ]);

        Ok(self.client.call("account_tx", params).await?)
    }
}

impl<C: Client + Clone> ChainStaking for XRPClient<C> {}

impl<C: Client + Clone> ChainPerpetual for XRPClient<C> {}

impl<C: Client + Clone> ChainAddressStatus for XRPClient<C> {}

impl<C: Client + Clone> chain_traits::ChainAccount for XRPClient<C> {}

impl<C: Client + Clone> ChainTraits for XRPClient<C> {}

impl<C: Client + Clone> ChainProvider for XRPClient<C> {
    fn get_chain(&self) -> Chain {
        self.chain
    }
}
