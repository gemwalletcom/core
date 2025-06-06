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
    pub fn new(url: String) -> Self {
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
            .parse::<i64>()?)
    }

    pub async fn get_transactions_by_block_number(&self, block_number: i64) -> Result<Digests, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!({
                "filter": {
                    "Checkpoint": block_number.to_string()
                },
                "options": {
                    "showEffects": true,
                    "showBalanceChanges": true,
                    "showEvents": true,
                    "showInput": false,
                }
            }),
            json!(null),
            json!(50),
        ];

        Ok(self.client.request("suix_queryTransactionBlocks", params).await?)
    }

    pub async fn get_coin_metadata(&self, token_id: String) -> Result<CoinMetadata, Box<dyn Error + Send + Sync>> {
        Ok(self.client.request("suix_getCoinMetadata", rpc_params!(token_id.clone())).await?)
    }

    pub async fn get_all_balances(&self, address: String) -> Result<Vec<Balance>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.request("suix_getAllBalances", rpc_params!(address)).await?)
    }
}
