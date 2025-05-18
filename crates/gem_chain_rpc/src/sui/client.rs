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
    model::{Balance, CoinMetadata, Digests},
};

pub const SUI_STAKE_EVENT: &str = "0x3::validator::StakingRequestEvent";
pub const SUI_UNSTAKE_EVENT: &str = "0x3::validator::UnstakingRequestEvent";

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

    pub fn get_fee(&self, gas_used: super::model::GasUsed) -> num_bigint::BigUint {
        SuiMapper::get_fee(gas_used)
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block: String = self.client.request("sui_getLatestCheckpointSequenceNumber", rpc_params![]).await?;
        Ok(block.parse::<i64>()?)
    }

    pub async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let digests = self.get_transactions_by_block_number(block_number).await?;
        let transactions = digests
            .data
            .into_iter()
            .flat_map(|x| SuiMapper::map_transaction(self.get_chain(), x, block_number))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }

    pub async fn get_transactions_by_block_number(&self, block_number: i64) -> Result<Digests, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!({
                "filter": {
                    "Checkpoint": block_number.to_string()
                },
                "options": {
                    "showEffects": true,
                    "showInput": false,
                    "showBalanceChanges":  true,
                    "showEvents": true
                }
            }),
            json!(null),
            json!(50),
            json!(true),
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
