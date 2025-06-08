use std::error::Error;

use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use primitives::chain::Chain;
use serde_json::json;

use super::{
    mapper::SuiMapper,
    model::{Balance, CoinMetadata, Digests, GasUsed},
};

pub struct SuiClient {
    client: HttpClient,
}

impl SuiClient {
    pub fn new(url: &str) -> Self {
        let client = HttpClientBuilder::default().build(url).unwrap();

        Self { client }
    }

    pub fn get_chain(&self) -> Chain {
        Chain::Sui
    }

    pub fn get_fee(&self, gas_used: GasUsed) -> num_bigint::BigUint {
        SuiMapper::get_fee(gas_used)
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .request::<String, _>("sui_getLatestCheckpointSequenceNumber", rpc_params![])
            .await?
            .parse()?)
    }

    async fn query_transaction_blocks(&self, filter: serde_json::Value) -> Result<Digests, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!({
                "filter": filter,
                "options": {
                    "showEffects": true,
                    "showBalanceChanges": true,
                    "showEvents": true,
                    "showInput": false,
                }
            }),
            json!(null),
            json!(100),
            json!(true),
        ];
        Ok(self.client.request("suix_queryTransactionBlocks", params).await?)
    }

    pub async fn get_transactions_by_block_number(&self, block_number: i64) -> Result<Digests, Box<dyn Error + Send + Sync>> {
        let filter = json!({
            "Checkpoint": block_number.to_string()
        });
        self.query_transaction_blocks(filter).await
    }

    pub async fn get_transactions_by_address(&self, address: String) -> Result<Digests, Box<dyn Error + Send + Sync>> {
        let filter = json!({
            "FromAddress": address
        });
        self.query_transaction_blocks(filter).await
    }

    pub async fn get_coin_metadata(&self, token_id: String) -> Result<CoinMetadata, Box<dyn Error + Send + Sync>> {
        Ok(self.client.request("suix_getCoinMetadata", rpc_params!(token_id.clone())).await?)
    }

    pub async fn get_all_balances(&self, address: String) -> Result<Vec<Balance>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.request("suix_getAllBalances", rpc_params!(address)).await?)
    }
}
